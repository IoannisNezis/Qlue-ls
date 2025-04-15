use std::io::{self, BufReader, Read};

pub(super) struct StdioMessages {
    bytes: std::io::Bytes<BufReader<std::io::Stdin>>,
    buffer: Vec<u8>,
}

impl StdioMessages {
    pub(super) fn new() -> Self {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let bytes = reader.bytes();
        Self {
            bytes,
            buffer: vec![],
        }
    }

    fn read_next_message(&mut self) -> Option<String> {
        loop {
            match self.bytes.next()? {
                Ok(byte) => {
                    self.buffer.push(byte);
                }
                Err(error) => {
                    log::error!("Error while reading byte: {}", error);
                    return None;
                }
            }
            // NOTE:: Wait for HEADER to end.
            if self.buffer.ends_with(b"\r\n\r\n") {
                let cl_slice = self
                    .buffer
                    .get(16..(self.buffer.len() - 4))
                    .expect("Header does not have a 'Content-Length: '");
                let cl_string =
                    String::from_utf8(cl_slice.to_vec().clone()).expect("Invalid UTF-8 data");
                let content_length: u32 =
                    cl_string.parse().expect("Failed to parse Content-Length");
                self.buffer.clear();
                // NOTE: READ next x bytes
                for ele in 0..content_length {
                    match self.bytes.next()? {
                        Ok(byte) => {
                            self.buffer.push(byte);
                        }
                        Err(err) => {
                            log::error!(
                                "Error {} occured while reading byte {} of {}, clearing buffer",
                                err,
                                ele,
                                content_length
                            );
                            self.buffer.clear();
                            break;
                        }
                    }
                }
                let message = String::from_utf8(self.buffer.clone()).ok()?;
                self.buffer.clear();
                return Some(message);
            }
        }
    }
}

impl Iterator for StdioMessages {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.read_next_message()
    }
}
