use super::TextDocumentContentChangeEvent;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use text_size::{TextRange, TextSize};

pub type DocumentUri = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentItem {
    pub uri: DocumentUri,
    language_id: String,
    version: u32,
    pub text: String,
}

impl TextDocumentItem {
    pub(crate) fn new(uri: &str, text: &str) -> TextDocumentItem {
        TextDocumentItem {
            uri: uri.to_string(),
            text: text.to_string(),
            language_id: "sparql".to_string(),
            version: 0,
        }
    }

    pub(crate) fn apply_text_edits(&mut self, mut text_edits: Vec<TextEdit>) {
        // NOTE: Sort text edit in "reverse" order.
        // When a text edit is applied all text behind the edit is shifted.
        // That means all text edits after the applied text edit become invalid.
        // To encounter this problem the edits are applied in reverse order.
        text_edits.sort_by(|a, b| {
            b.range
                .start
                .cmp(&a.range.start)
                .then_with(|| b.range.end.cmp(&a.range.end))
        });

        // NOTE: Compute the inline-utf16_index of each line ending.
        // For example the text:
        // ```
        // abc
        // a
        // abcd
        // ```
        // the inline-utf16_index of each line endig would be:
        // [3,1,4]
        let mut line_breaks: Vec<usize> = Vec::new();
        let mut utf_16_counter = 0;
        for char in self.text.chars() {
            if char == '\n' {
                line_breaks.push(utf_16_counter);
                utf_16_counter = 0;
            } else {
                utf_16_counter += char.len_utf16();
            }
        }

        let mut cursor = if !matches!(self.text.chars().last(), Some('\n')) {
            Position::new((line_breaks.len()) as u32, utf_16_counter as u32)
        } else {
            Position::new(line_breaks.len() as u32, 0)
        };

        let mut edits = text_edits.into_iter().peekable();
        let mut current_edit_end_byte_offset = self.text.len();
        let mut byte_offset = self.text.len();
        let chars: Vec<char> = self.text.chars().collect();
        let mut char_offset = chars.len();
        while let Some(edit) = edits.peek() {
            assert!(
                edit.range.start <= cursor,
                "A edit was missed when appying edits. The next edit start position: {}, cursor position: {cursor}",
                edit.range.start
            );
            if edit.range.end == cursor {
                current_edit_end_byte_offset = byte_offset;
            }
            if edit.range.start == cursor {
                self.text
                    .replace_range(byte_offset..current_edit_end_byte_offset, &edit.new_text);
                edits.next();
                continue;
            }
            if char_offset > 0 {
                let char = chars[char_offset - 1];
                if char == '\n' {
                    cursor.line -= 1;
                    cursor.character = line_breaks.pop().unwrap() as u32;
                } else {
                    cursor.character -= char.len_utf16() as u32;
                }
                byte_offset -= char.len_utf8();
                char_offset -= 1;
            }
        }
        if !matches!(self.text.chars().last(), Some('\n')) {
            self.text.push('\n');
        }
    }

    pub(crate) fn increase_version(&mut self) {
        self.version += 1;
    }

    pub(crate) fn version(&self) -> u32 {
        self.version
    }

    // NOTE: This function applies `TextDocumentContentChangeEvent[]`.
    // These differe in behaviour from `TextEdit[]` in the following way.
    // `TextEdit[]` describes a transformation from a document in state A to state B
    // without any intermediate state. So each TextEdit is refering to a range in the original
    // document, unaffected by previous edits. In other words, the order in which edits are applied
    // does not matter!
    // For `TextDocumentContentChangeEvent[]` the order does matter, they have to be applied
    // in order and each change moves the document into a new state.
    // The fuction 'apply_text_edits' is meant for `TextEdit[]` and this one for `TextDocumentContentChangeEvent`
    pub(crate) fn apply_content_changes(
        &mut self,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) {
        let mut changes = content_changes.into_iter().peekable();
        // NOTE: This is refering to a LSP position in the text.
        let mut cursor: Position = Position::new(0, 0);
        // NOTE: This is refering to a utf-8 byte offset in the text.
        let mut byte_offset: usize = 0;
        // NOTE: This is refering to a utf-8 byte offset in the text.
        let mut current_change_start_byte_offset: usize = 0;
        while let Some(change) = changes.peek() {
            assert!(
                change.range.start <= change.range.end,
                "received a invalid change: {change:?}"
            );
            assert!(
                change.range.end >= cursor,
                "A change was missed when applying changes. The next change end position: {}, cursor position: {cursor}",
                change.range.end
            );
            if change.range.start == cursor {
                current_change_start_byte_offset = byte_offset;
            }
            if change.range.end == cursor {
                self.text
                    .replace_range(current_change_start_byte_offset..byte_offset, &change.text);
                // NOTE: The cursor is now at the end of the edit.
                // Therefor its position in the document needs to be corrected.
                cursor = change.range.start;
                byte_offset = current_change_start_byte_offset;
                changes.next();
                // NOTE: The changes don't have to be ordered.
                // If the next change is infront of the cursor, the cursor has to be reset.
                if changes
                    .peek()
                    .is_some_and(|change| change.range.start < cursor)
                {
                    cursor = Position::new(0, 0);
                    byte_offset = 0;
                }
                continue;
            }
            let chr = self.text[byte_offset..]
                .chars()
                .next()
                .expect(&format!("{} should be a valid utf-8 char offset and there should be a next character, since there is an unapplied change", byte_offset));
            match chr {
                '\n' => {
                    cursor.line += 1;
                    cursor.character = 0;
                    byte_offset += chr.len_utf8();
                }
                '\r' if self.text[byte_offset..].starts_with("\r\n") => {
                    cursor.line += 1;
                    cursor.character = 0;
                    byte_offset += 2;
                }
                _ => {
                    byte_offset += chr.len_utf8();
                    cursor.character += chr.len_utf16() as u32;
                }
            };
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VersionedTextDocumentIdentifier {
    #[serde(flatten)]
    pub base: TextDocumentIdentifier,
    version: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TextDocumentIdentifier {
    pub uri: Uri,
}

type Uri = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

// NOTE: By default based on a UTF-16 string representation!
impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }

    /// Convert UTF-8 byte offset in a text into UTF-16 based line/column Position
    ///
    /// Returns None if:
    /// - offset is outside the given str
    /// - offset is not on the border of a UTF-8 codepoint
    pub fn from_byte_index(offset: TextSize, text: &str) -> Option<Self> {
        let offset_usize: usize = offset.into();
        let mut offset_count = 0;
        let mut position = Self::new(0, 0);
        let mut chars = text.chars().peekable();
        while let Some(chr) = chars.next()
            && offset_count < offset_usize
        {
            match chr {
                '\n' => {
                    position.line += 1;
                    position.character = 0;
                }
                _ => {
                    position.character += chr.len_utf16() as u32;
                }
            }
            offset_count += chr.len_utf8();
        }
        // NOTE: the byte offset MUST be at the start or end of a UTF-8 char.
        // https://datatracker.ietf.org/doc/html/rfc2119
        (offset_count == offset_usize).then_some(position)
    }

    /// Converts a UTF-16 based position within a string to a byte index.
    ///
    /// # Arguments
    ///
    /// * `text` - A reference to the string in which the position is calculated.
    ///
    /// # Returns
    ///
    /// * `Option<usize>` - The byte index corresponding to the UTF-16 position
    ///   if the position is valid. Returns `None` if the position is out of bounds
    ///   or if the conversion cannot be performed.
    ///
    /// # Details
    ///
    /// This function takes into account the difference between UTF-8 and UTF-16
    /// representations. In UTF-16, some characters, such as those outside the
    /// Basic Multilingual Plane (e.g., emoji or certain CJK characters), are
    /// represented as surrogate pairs, which occupy two 16-bit code units.
    /// In contrast, UTF-8 uses a variable-length encoding where these same
    /// characters can take up to four bytes.
    ///
    /// The function ensures that the given UTF-16 position is correctly
    /// mapped to its corresponding byte index in the UTF-8 encoded string,
    /// preserving the integrity of multi-byte characters.
    ///
    /// # Caveats
    ///
    /// * If `text` contains invalid UTF-8 sequences, the behavior of this function
    ///   is undefined.
    /// * Ensure the provided UTF-16 position aligns with the logical structure of
    ///   the string.
    pub fn byte_index(&self, text: &str) -> Option<TextSize> {
        if self.line == 0 && self.character == 0 && text.is_empty() {
            return Some(TextSize::new(0));
        }
        let mut byte_index: usize = 0;
        let mut line_idx = 0;
        let mut chars = text.chars().peekable();
        while line_idx < self.line {
            match chars.next() {
                Some('\n') => {
                    line_idx += 1;
                    byte_index += 1;
                }
                Some('\r') if chars.peek().is_some_and(|chr| chr == &'\n') => {
                    line_idx += 1;
                    byte_index += 2;
                    chars.next();
                }
                Some(chr) => {
                    byte_index += chr.len_utf8();
                }
                None => {
                    return None;
                }
            }
        }
        let mut utf16_index = 0;
        while utf16_index < self.character as usize {
            let char = chars.next()?;
            if char == '\n' || (char == '\r' && chars.next().is_some_and(|chr| chr == '\n')) {
                return None;
            }
            byte_index += char.len_utf8();
            utf16_index += char.len_utf16();
        }
        Some(TextSize::new(byte_index as u32))
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Equal => self.character.cmp(&other.character),
            x => x,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:0>2}:{:0>2}", self.line, self.character)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#range
// NOTE: Positions are zero based.
// NOTE: The end position is exclusive.
// NOTE: To include line ending character(s), set end position to the start of next line.
/// LSP text range (UTF-16 based)
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start_line: u32, start_character: u32, end_line: u32, end_character: u32) -> Self {
        Self {
            start: Position::new(start_line, start_character),
            end: Position::new(end_line, end_character),
        }
    }

    pub fn empty(position: Position) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    pub fn to_byte_index_range(&self, text: &str) -> Option<TextRange> {
        match (self.start.byte_index(text), self.end.byte_index(text)) {
            (Some(from), Some(to)) => Some(TextRange::new(from, to)),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn overlaps(&self, other: &Range) -> bool {
        self.start < other.end && self.end > other.start
    }

    fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub(crate) fn from_byte_offset_range(range: text_size::TextRange, text: &str) -> Option<Range> {
        Some(Range {
            start: Position::from_byte_index(range.start().into(), text)?,
            end: Position::from_byte_index(range.end().into(), text)?,
        })
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}-{}", self.start, self.end))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

impl TextEdit {
    pub fn new(range: Range, new_text: &str) -> Self {
        Self {
            range,
            new_text: new_text.to_string(),
        }
    }

    #[cfg(test)]
    pub fn overlaps(&self, other: &TextEdit) -> bool {
        self.range.overlaps(&other.range)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.range.is_empty() && self.new_text.is_empty()
    }
}

impl Display for TextEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "{} \"{}\"",
            self.range,
            self.new_text.replace(" ", "␣").replace("\n", "\\n")
        ))
    }
}

#[cfg(test)]
mod tests {

    use indoc::indoc;
    use text_size::{TextRange, TextSize};

    use crate::server::lsp::{
        TextDocumentContentChangeEvent,
        textdocument::{Position, Range, TextEdit},
    };

    use super::TextDocumentItem;

    #[test]
    fn byte_index_to_position() {
        let s = "aä😀\n123ä\n";
        assert_eq!(
            Position::from_byte_index(0.into(), s).unwrap(),
            Position::new(0, 0)
        );

        assert_eq!(
            Position::from_byte_index(1.into(), s).unwrap(),
            Position::new(0, 1)
        );
        assert_eq!(
            Position::from_byte_index(3.into(), s).unwrap(),
            Position::new(0, 2)
        );
        assert_eq!(
            Position::from_byte_index(7.into(), s).unwrap(),
            Position::new(0, 4)
        );
        assert_eq!(
            Position::from_byte_index(8.into(), s).unwrap(),
            Position::new(1, 0)
        );
        assert_eq!(
            Position::from_byte_index(9.into(), s).unwrap(),
            Position::new(1, 1)
        );
        assert_eq!(
            Position::from_byte_index(10.into(), s).unwrap(),
            Position::new(1, 2)
        );
        assert_eq!(
            Position::from_byte_index(13.into(), s).unwrap(),
            Position::new(1, 4)
        );
        assert_eq!(
            Position::from_byte_index(14.into(), s).unwrap(),
            Position::new(2, 0)
        );
        assert_eq!(Position::from_byte_index(15.into(), s), None);
        assert_eq!(Position::from_byte_index(2.into(), s), None);
    }

    // #[test]
    // fn translate_utf8_utf16() {
    //     let s = "aä😀\n".to_string();
    //     let mut p0 = Position::new(0, 0);
    //     p0.translate_to_utf16_encoding(&s).unwrap();
    //     assert_eq!(p0, Position::new(0, 0));
    //
    //     let mut p1 = Position::new(0, 1);
    //     p1.translate_to_utf16_encoding(&s).unwrap();
    //     assert_eq!(p1, Position::new(0, 1));
    //
    //     let mut p2 = Position::new(0, 3);
    //     p2.translate_to_utf16_encoding(&s).unwrap();
    //     assert_eq!(p2, Position::new(0, 2));
    //
    //     let mut p3 = Position::new(0, 7);
    //     p3.translate_to_utf16_encoding(&s).unwrap();
    //     assert_eq!(p3, Position::new(0, 4));
    //
    //     let mut p4 = Position::new(1, 0);
    //     p4.translate_to_utf16_encoding(&s).unwrap();
    //     assert_eq!(p4, Position::new(1, 0));

    #[test]
    fn changes() {
        let mut document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "".to_string(),
        };
        assert_eq!(document.text, "");
        document.apply_text_edits(vec![TextEdit {
            new_text: "SELECT ".to_string(),
            range: Range::new(0, 0, 0, 0),
        }]);
        assert_eq!(document.text, "SELECT \n");
        document.apply_text_edits(vec![
            TextEdit {
                new_text: " DISTINCT".to_string(),
                range: Range::new(0, 6, 0, 6),
            },
            TextEdit {
                new_text: "* WHERE{\n  ?s ?p ?o\n}".to_string(),
                range: Range::new(0, 7, 0, 7),
            },
        ]);
        assert_eq!(document.text, "SELECT DISTINCT * WHERE{\n  ?s ?p ?o\n}\n");
        document.apply_text_edits(vec![TextEdit {
            new_text: "select".to_string(),
            range: Range::new(0, 0, 0, 6),
        }]);
        assert_eq!(document.text, "select DISTINCT * WHERE{\n  ?s ?p ?o\n}\n");
        document.apply_text_edits(vec![
            TextEdit {
                new_text: "".to_string(),
                range: Range::new(1, 10, 2, 0),
            },
            TextEdit {
                new_text: "".to_string(),
                range: Range::new(0, 24, 1, 1),
            },
        ]);
        assert_eq!(document.text, "select DISTINCT * WHERE{ ?s ?p ?o}\n");
        document.apply_text_edits(vec![
            TextEdit {
                new_text: "ns1:dings".to_string(),
                range: Range::new(0, 25, 0, 27),
            },
            TextEdit {
                new_text: "PREFIX ns1: <iri>\n".to_string(),
                range: Range::new(0, 0, 0, 0),
            },
        ]);
        assert_eq!(
            document.text,
            "PREFIX ns1: <iri>\nselect DISTINCT * WHERE{ ns1:dings ?p ?o}\n"
        );
        document.apply_text_edits(vec![
            TextEdit {
                new_text: "".to_string(),
                range: Range::new(1, 10, 2, 0),
            },
            TextEdit {
                new_text: "".to_string(),
                range: Range::new(0, 0, 1, 10),
            },
        ]);
        assert_eq!(document.text, "\n");
    }

    #[test]
    fn apply_change() {
        let mut document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "\n".to_string(),
        };
        let change = TextEdit {
            new_text: "dings".to_string(),
            range: Range::new(0, 0, 0, 0),
        };
        document.apply_text_edits(vec![change]);
        assert_eq!(document.text, "dings\n");
    }

    #[test]
    fn position_to_byte_index() {
        let text = "aä�𐀀\n".to_string();
        // assert_eq!(
        //     Position::new(0, 0).byte_index(&text),
        //     Some(TextSize::new(0))
        // );
        // assert_eq!(
        //     Position::new(0, 1).byte_index(&text),
        //     Some(TextSize::new(1))
        // );
        // assert_eq!(
        //     Position::new(0, 2).byte_index(&text),
        //     Some(TextSize::new(3))
        // );
        // assert_eq!(
        //     Position::new(0, 3).byte_index(&text),
        //     Some(TextSize::new(6))
        // );
        // assert_eq!(
        //     Position::new(0, 5).byte_index(&text),
        //     Some(TextSize::new(10))
        // );
        // assert_eq!(
        //     Position::new(1, 0).byte_index(&text),
        //     Some(TextSize::new(11))
        // );
        assert_eq!(Position::new(2, 0).byte_index(&text), None);
    }

    #[test]
    fn range_to_byte_index_range() {
        let text = indoc!(
            "12345
             12345
             12345
             "
        )
        .to_string();
        assert_eq!(
            Range::new(0, 5, 1, 1).to_byte_index_range(&text),
            Some(TextRange::new(TextSize::new(5), TextSize::new(7)))
        );
        let range = Range::new(1, 0, 2, 0);
        let pos = range.start;
        assert_eq!(pos.byte_index(&text), Some(TextSize::new(6)));
        assert_eq!(
            Range::new(1, 0, 2, 0).to_byte_index_range(&text),
            Some(TextRange::new(TextSize::new(6), TextSize::new(12)))
        );
        assert_eq!(
            Range::new(0, 0, 3, 0).to_byte_index_range(&text),
            Some(TextRange::new(TextSize::new(0), TextSize::new(18)))
        );

        assert_eq!(Range::new(0, 0, 3, 1).to_byte_index_range(&text), None);
        assert_eq!(Range::new(0, 0, 1, 10).to_byte_index_range(&text), None);
    }

    #[test]
    fn no_changes() {
        let changes: Vec<TextEdit> = vec![];
        let mut document: TextDocumentItem = TextDocumentItem {
            uri: "file:///dings".to_string(),
            language_id: "foo".to_string(),
            version: 1,
            text: "hello world\n".to_string(),
        };
        document.apply_text_edits(changes);
        assert_eq!(document.text, "hello world\n");
    }

    #[test]
    fn overlap() {
        let a = Range::new(1, 1, 2, 2); //      >----<
        let b = Range::new(1, 10, 2, 5); //        >----<
        let c = Range::new(0, 0, 1, 10); //   >--<
        let d = Range::new(1, 10, 2, 6); //         >-<
        let e = Range::new(2, 6, 2, 7); //                >--<

        assert!(a.overlaps(&b));
        assert!(a.overlaps(&c));
        assert!(a.overlaps(&d));
        assert!(!a.overlaps(&e));

        assert!(b.overlaps(&a));
        assert!(!b.overlaps(&c));
        assert!(b.overlaps(&d));
        assert!(!b.overlaps(&e));

        assert!(c.overlaps(&a));
        assert!(!c.overlaps(&b));
        assert!(!c.overlaps(&d));
        assert!(!c.overlaps(&e));

        assert!(d.overlaps(&a));
        assert!(d.overlaps(&b));
        assert!(!d.overlaps(&c));
        assert!(!d.overlaps(&e));

        assert!(!e.overlaps(&a));
        assert!(!e.overlaps(&b));
        assert!(!e.overlaps(&c));
        assert!(!e.overlaps(&d));
    }

    #[test]
    fn multiline_delete_edit() {
        // Reproduces the bug where deleting multiple lines causes panic
        let initial_text = indoc!(
            "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
             PREFIX rml: <http://w3id.org/rml/>
             SELECT
               *
             WHERE {
               ?c a ex:TesterCluster;
                  tester:hasQuestionEdit ?qe ;
                  ex:usesDataFrom ?endpoint .
             }
             "
        );

        let mut document = TextDocumentItem::new("file:///test.rq", initial_text);

        // Delete multiple lines to leave just: "?c  \n}"
        let edit = TextEdit {
            range: Range::new(5, 5, 7, 31), // Delete from middle of line 6 to end of line 8
            new_text: "".to_string(),
        };

        document.apply_text_edits(vec![edit]);
        println!("{}", document.text);

        // Should not panic
        assert!(document.text.contains("?c"));
        assert!(document.text.contains("}"));
    }

    #[test]
    fn byte_index_with_crlf() {
        // Test that byte_index handles CRLF line endings correctly
        let text = "foo\r\nbar\r\nbaz\r\n";

        // Line 0, char 0 should be at byte 0
        assert_eq!(Position::new(0, 0).byte_index(text), Some(TextSize::new(0)));

        // Line 0, char 3 should be at byte 3 (end of "foo")
        assert_eq!(Position::new(0, 3).byte_index(text), Some(TextSize::new(3)));

        // Line 1, char 0 should be at byte 5 (after "foo\r\n")
        // But current implementation returns byte 4 (incorrect!)
        assert_eq!(Position::new(1, 0).byte_index(text), Some(TextSize::new(5)));

        // Line 2, char 0 should be at byte 10 (after "foo\r\nbar\r\n")
        assert_eq!(
            Position::new(2, 0).byte_index(text),
            Some(TextSize::new(10))
        );
    }

    #[test]
    fn overlapping_edits_same_start() {
        // Reproduces bug where multiple edits with the same start position
        // but different end positions cause invalid byte offsets
        let initial_text = indoc!(
            "line 0
             line 1
             line 2
             line 3
             line 4
             line 5
             "
        );

        let mut document = TextDocumentItem::new("file:///test.txt", initial_text);

        // Two overlapping edits that both start at line 2:0
        let edits = vec![
            TextEdit {
                range: Range::new(2, 0, 4, 0), // Delete lines 2-3
                new_text: "".to_string(),
            },
            TextEdit {
                range: Range::new(2, 0, 3, 0), // Delete line 2 (overlaps with above)
                new_text: "".to_string(),
            },
        ];

        // Should not panic
        document.apply_text_edits(edits);

        // Document should have some content left
        assert!(document.text.len() > 0);
    }

    #[test]
    fn content_change_event_1() {
        let initial_text = "\n";
        let mut document = TextDocumentItem::new("file:///test.txt", initial_text);
        document.apply_content_changes(vec![TextDocumentContentChangeEvent {
            range: Range::new(0, 0, 0, 0),
            text: "a".to_string(),
        }]);
        document.apply_content_changes(vec![
            TextDocumentContentChangeEvent {
                range: Range::new(0, 1, 0, 1),
                text: "b".to_string(),
            },
            TextDocumentContentChangeEvent {
                range: Range::new(0, 2, 0, 2),
                text: "c".to_string(),
            },
        ]);
        assert_eq!(document.text, "abc\n");
    }

    #[test]
    fn content_change_event_2() {
        let initial_text = indoc!(
            "SELECT  *  WHERE {
               ?s ?p ?o
            "
        );
        let mut document = TextDocumentItem::new("file:///test.txt", initial_text);
        document.apply_content_changes(vec![
            TextDocumentContentChangeEvent {
                range: Range::new(0, 10, 0, 11),
                text: "".to_string(),
            },
            TextDocumentContentChangeEvent {
                range: Range::new(0, 7, 0, 8),
                text: "".to_string(),
            },
        ]);
        assert_eq!(
            document.text,
            indoc! {
            "SELECT * WHERE {
               ?s ?p ?o
            "}
        );
    }

    #[test]
    fn content_change_event_3() {
        //                  01234567890123456789
        let initial_text = "SELECT * WHERE { r\n";

        let mut document = TextDocumentItem::new("file:///test.txt", initial_text);
        document.apply_content_changes(vec![
            TextDocumentContentChangeEvent {
                range: Range::new(0, 16, 0, 18),
                text: "\n\n\n\n\nr".to_string(),
            },
            TextDocumentContentChangeEvent {
                range: Range::new(5, 1, 5, 1),
                text: "".to_string(),
            },
        ]);
    }
    #[test]
    fn content_change_event_4() {
        let initial_text = indoc! {"
            SELECT * WHERE {




            r
            "
        };

        let mut document = TextDocumentItem::new("file:///test.txt", initial_text);
        document.apply_content_changes(vec![
            TextDocumentContentChangeEvent {
                range: Range::new(5, 1, 5, 1),
                text: "".to_string(),
            },
            TextDocumentContentChangeEvent {
                range: Range::new(0, 16, 5, 1),
                text: " r".to_string(),
            },
            TextDocumentContentChangeEvent {
                range: Range::new(0, 18, 0, 18),
                text: "".to_string(),
            },
            TextDocumentContentChangeEvent {
                range: Range::new(0, 18, 0, 18),
                text: "".to_string(),
            },
        ]);
    }
}
