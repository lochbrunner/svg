use std::iter::Peekable;
use std::str::Chars;

pub struct Reader<'s> {
    text: &'s str,

    line: usize,
    column: usize,
    offset: usize,

    cursor: Peekable<Chars<'s>>,
}

impl<'s> Reader<'s> {
    #[inline]
    pub fn new(text: &'s str) -> Reader<'s> {
        Reader {
            text: text,

            line: 1,
            column: 1,
            offset: 0,

            cursor: text.chars().peekable(),
        }
    }

    pub fn capture<F>(&mut self, block: F) -> Option<&str> where F: Fn(&mut Reader<'s>) {
        let start = self.offset;
        block(self);
        let end = self.offset;

        if end > start {
            Some(&self.text[start..end])
        } else {
            None
        }
    }

    pub fn consume_while<F>(&mut self, check: F) where F: Fn(char) -> bool {
        loop {
            match self.peek() {
                Some(c) => {
                    if check(c) {
                        self.next();
                    } else {
                        break;
                    }
                },
                _ => break,
            }
        }
    }

    #[inline]
    pub fn consume_any(&mut self, chars: &str) {
        self.consume_while(|c| chars.contains_char(c))
    }

    #[inline]
    pub fn consume_until_char(&mut self, target: char) {
        self.consume_while(|c| c != target)
    }

    #[inline]
    pub fn consume_digits(&mut self) {
        self.consume_while(|c| c >= '0' && c <= '9')
    }

    #[inline]
    pub fn consume_whitespace(&mut self) {
        self.consume_any(" \t\n")
    }

    #[inline]
    pub fn peek(&mut self) -> Option<char> {
        self.cursor.peek().and_then(|&c| Some(c))
    }

    #[inline]
    pub fn position(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    pub fn read_char(&mut self, target: char) -> Option<char> {
        match self.peek() {
            Some(c) if c == target => {
                self.next();
                Some(c)
            },
            _ => None
        }
    }

    /// http://www.w3.org/TR/REC-xml/#NT-NameStartChar
    pub fn read_name_start_char(&mut self) -> Option<char> {
        match self.peek() {
            Some(c) => match c {
                ':' |
                'A'...'Z' |
                '_' |
                'a'...'z' |
                '\u{C0}'...'\u{D6}' |
                '\u{D8}'...'\u{F6}' |
                '\u{F8}'...'\u{2FF}' |
                '\u{370}'...'\u{37D}' |
                '\u{37F}'...'\u{1FFF}' |
                '\u{200C}'...'\u{200D}' |
                '\u{2070}'...'\u{218F}' |
                '\u{2C00}'...'\u{2FEF}' |
                '\u{3001}'...'\u{D7FF}' |
                '\u{F900}'...'\u{FDCF}' |
                '\u{FDF0}'...'\u{FFFD}' |
                '\u{10000}'...'\u{EFFFF}' => {
                    self.next();
                    Some(c)
                },
                _ => None,
            },
            _ => None
        }
    }

    /// http://www.w3.org/TR/REC-xml/#NT-NameChar
    pub fn read_name_char(&mut self) -> Option<char> {
        self.read_name_start_char().or_else(|| {
            match self.peek() {
                Some(c) => match c {
                    '-' |
                    '.' |
                    '0'...'9' |
                    '\u{B7}' |
                    '\u{0300}'...'\u{036F}' |
                    '\u{203F}'...'\u{2040}' => {
                        self.next();
                        Some(c)
                    },
                    _ => None,
                },
                _ => None,
            }
        })
    }

    /// http://www.w3.org/TR/REC-xml/#NT-Name
    pub fn read_name(&mut self) -> Option<&str> {
        self.capture(|reader| {
            match reader.read_name_start_char() {
                Some(_) => {
                    loop {
                        match reader.read_name_char() {
                            Some(_) => {},
                            _ => break,
                        }
                    }
                },
                _ => {},
            }
        })
    }
}

impl<'s> Iterator for Reader<'s> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.cursor.next() {
            Some(c) => {
                if c == '\n' {
                    self.line += 1;
                    self.column = 1;
                } else {
                    self.column += 1;
                }
                self.offset += 1;
                Some(c)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn capture() {
        let mut reader = Reader::new("abcdefg");

        reader.consume_any("ab");
        let text = reader.capture(|reader| {
            reader.consume_any("cde");
        });

        assert_eq!(text.unwrap(), "cde");
    }

    #[test]
    fn consume_whitespace() {
        let mut reader = Reader::new(" \t  \n\n  \tm ");

        reader.consume_whitespace();

        assert_eq!(reader.line, 3);
        assert_eq!(reader.column, 4);
        assert_eq!(reader.offset, 9);
    }

    #[test]
    fn read_name() {
        macro_rules! test(
            ($text:expr, $name:expr) => ({
                let mut reader = Reader::new($text);
                assert_eq!(reader.read_name().unwrap(), $name);
            });
        );

        test!("foo", "foo");
        test!("foo bar", "foo");
        test!("foo42 bar", "foo42");
        test!("foo-bar baz", "foo-bar");
        test!("foo/", "foo");

        macro_rules! test(
            ($text:expr) => ({
                let mut reader = Reader::new($text);
                assert!(reader.read_name().is_none());
            });
        );

        test!(" foo");
        test!("!foo");
        test!("<foo");
        test!("?foo");
    }
}