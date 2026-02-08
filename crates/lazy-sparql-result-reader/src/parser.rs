use serde::{Deserialize, Serialize};

use crate::sparql::{Binding, Header, Meta};

pub struct Parser {
    scanner_state: ScannerState,
    input_buffer: Vec<u8>,
    batch_size: usize,
    binding_buffer: Vec<Binding>,
    binding_counter: usize,
    limit: Option<usize>,
    offset: usize,
}

impl Parser {
    pub fn new(batch_size: usize, limit: Option<usize>, offset: usize) -> Self {
        Self {
            scanner_state: ScannerState::ReadingHead,
            input_buffer: Vec::new(),
            batch_size,
            binding_buffer: Vec::with_capacity(batch_size),
            binding_counter: 0,
            limit,
            offset,
        }
    }

    /// Returins the remaining bindings, consuming the parser.
    pub fn flush(self) -> Option<PartialResult> {
        (!self.binding_buffer.is_empty()).then_some(PartialResult::Bindings(self.binding_buffer))
    }
}

#[derive(Debug, Clone)]
enum ScannerState {
    ReadingHead,
    SearchingBindings,
    SearchingBinding,
    ReadingBinding(u8),
    ReadingString(Box<ScannerState>),
    ReadingStringEscaped(Box<ScannerState>),
    SearchchingMeta,
    ReadingMeta,
    Done,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PartialResult {
    Header(Header),
    Bindings(Vec<Binding>),
    Meta(Meta),
}

impl Parser {
    pub fn read_byte(&mut self, byte: u8) -> Result<Option<PartialResult>, serde_json::Error> {
        let current_state = self.scanner_state.clone();
        // NOTE: the current char does not always have to be storred
        let store_char = {
            matches!(current_state, ScannerState::ReadingHead | ScannerState::ReadingMeta | ScannerState::ReadingString(_) | ScannerState::ReadingStringEscaped(_))
            ||
            // NOTE: Dont store if in reading binding, but not in read window.
            (matches!(current_state, ScannerState::ReadingBinding(_))
                && self.binding_counter >= self.offset && self .limit
                        .is_none_or(|limit| self.binding_counter - self.offset < limit))
        };
        // println!("{}", self.input_buffer);
        // log::info!(
        //     "limit: {:?}, offset: {}, count: {}, store: {store_char}",
        //     self.limit,
        //     self.offset,
        //     self.binding_counter
        // );
        if store_char {
            self.input_buffer.push(byte);
        }
        match (byte, current_state) {
            (b'}', ScannerState::ReadingHead) => {
                self.input_buffer.push(b'}');
                let header: Header = serde_json::from_slice(&self.input_buffer)?;
                self.scanner_state = ScannerState::SearchingBindings;
                return Ok(Some(PartialResult::Header(header)));
            }
            (b'}', ScannerState::ReadingBinding(1)) => {
                self.binding_counter += 1;
                self.scanner_state = ScannerState::SearchingBinding;
                if self.binding_counter > self.offset
                    && self
                        .limit
                        .is_none_or(|limit| self.binding_counter - self.offset <= (limit))
                {
                    let binding: Binding = serde_json::from_slice(&self.input_buffer)?;
                    self.binding_buffer.push(binding);
                    if self.binding_buffer.len() == self.batch_size {
                        let bindings = std::mem::take(&mut self.binding_buffer);
                        return Ok(Some(PartialResult::Bindings(bindings)));
                    }
                } else {
                    self.input_buffer.clear();
                }
            }
            (b'[', ScannerState::SearchingBindings) => {
                self.input_buffer.clear();
                self.scanner_state = ScannerState::SearchingBinding;
            }
            (b'{', ScannerState::SearchingBinding) => {
                self.input_buffer.clear();
                self.input_buffer.push(b'{');
                self.scanner_state = ScannerState::ReadingBinding(1);
            }
            (b'{', ScannerState::ReadingBinding(depth)) => {
                self.scanner_state = ScannerState::ReadingBinding(depth + 1);
            }
            (b'}', ScannerState::ReadingBinding(depth)) => {
                self.scanner_state = ScannerState::ReadingBinding(depth - 1);
            }
            (b'"', ScannerState::ReadingBinding(_) | ScannerState::ReadingHead) => {
                self.scanner_state =
                    ScannerState::ReadingString(Box::new(self.scanner_state.clone()));
            }
            (b'"', ScannerState::ReadingString(prev_state)) => {
                self.scanner_state = *prev_state;
            }
            (b'\\', ScannerState::ReadingString(prev_state)) => {
                self.scanner_state = ScannerState::ReadingStringEscaped(prev_state);
            }
            (_, ScannerState::ReadingStringEscaped(prev_state)) => {
                self.scanner_state = ScannerState::ReadingString(prev_state);
            }
            (b']', ScannerState::SearchingBinding) => {
                self.scanner_state = ScannerState::SearchchingMeta;
            }
            (b'{', ScannerState::SearchchingMeta) => {
                self.input_buffer.clear();
                self.input_buffer.push(b'{');
                self.scanner_state = ScannerState::ReadingMeta;
            }
            (b'}', ScannerState::ReadingMeta) => {
                self.scanner_state = ScannerState::Done;
                let meta: Meta = serde_json::from_slice(&self.input_buffer)?;
                return Ok(Some(PartialResult::Meta(meta)));
            }
            _ => {}
        };
        Ok(None)
    }
}

#[cfg(test)]
mod test {

    use crate::{
        parser::{Parser, PartialResult},
        sparql::{Bindings, Head, Header, Meta, SparqlResult},
    };

    #[test]
    fn parser_schema() {
        let input = r#"{"head":{"vars":[]},"results":{"bindings":[{"":{"type":"uri","value":""},"U*":{"type":"uri","value":"*\"","curie":""}},{"":{"type":"uri","value":""},"U*":{"type":"uri","value":"*\"","curie":""}}]},"meta":{"query-time-ms":0,"result-size-total":0}}"#;
        let serde_parsed_result: SparqlResult = serde_json::from_str(&input).unwrap();

        let mut parsed_result = SparqlResult {
            head: Head { vars: Vec::new() },
            results: Bindings {
                bindings: Vec::new(),
            },
            meta: Meta {
                query_time_ms: 0,
                result_size_total: 0,
            },
        };

        let mut parser = Parser::new(1, None, 0);
        for byte in input.as_bytes() {
            match parser.read_byte(*byte).expect("Input should be valid") {
                Some(PartialResult::Header(Header { head })) => parsed_result.head = head,
                Some(PartialResult::Bindings(bindings)) => {
                    parsed_result.results.bindings.extend(bindings);
                }
                Some(PartialResult::Meta(meta)) => parsed_result.meta = meta,
                None => {}
            }
        }

        assert_eq!(
            serde_parsed_result, parsed_result,
            "parser failed for this input:\n{input}"
        );
    }
}
