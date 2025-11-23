use std::marker::PhantomData;

use crate::types::LogStr;

pub struct Parser<'a> {
    source: *const u8,
    ptr: *const u8,
    end: *const u8,
    _marker: PhantomData<&'a u8>,
}

impl<'a> Parser<'a> {
    pub fn new(buffer: &'a [u8]) -> Parser<'a> {
        let ptr = buffer.as_ptr();
        let end = unsafe { ptr.add(buffer.len()) };
        Parser {
            source: ptr,
            ptr,
            end,
            _marker: PhantomData,
        }
    }

    pub fn position(&self) -> usize {
        unsafe { self.ptr.offset_from(self.source) as usize }
    }

    pub fn next(&mut self) -> Option<u8> {
        if self.ptr == self.end {
            None
        } else {
            let v = unsafe { *self.ptr };
            self.ptr = unsafe { self.ptr.add(1) };
            Some(v)
        }
    }

    pub fn skip(&mut self, count: usize) -> Option<()> {
        let new_ptr = unsafe { self.ptr.add(count) };
        if new_ptr > self.end {
            None
        } else {
            self.ptr = new_ptr;
            Some(())
        }
    }

    pub fn skip_to(&mut self, ch: u8) -> Option<()> {
        let len = unsafe { self.end.offset_from(self.ptr) } as usize;
        let haystack = unsafe { std::slice::from_raw_parts(self.ptr, len) };
        let i = memchr::memchr(ch, haystack)?;
        self.skip(i + 1)
    }

    pub fn skip_to2(&mut self, ch1: u8, ch2: u8) -> Option<()> {
        let len = unsafe { self.end.offset_from(self.ptr) } as usize;
        let haystack = unsafe { std::slice::from_raw_parts(self.ptr, len) };
        let i = memchr::memchr2(ch1, ch2, haystack)?;
        self.skip(i + 1)
    }

    pub fn peek(&self) -> Option<u8> {
        if self.ptr == self.end {
            None
        } else {
            let v = unsafe { *self.ptr };
            Some(v)
        }
    }

    pub fn parse_number<T>(&mut self, delimiter: char) -> Option<T>
    where
        T: Default + std::ops::Mul<T, Output = T> + std::ops::Add<T, Output = T> + From<u8>,
    {
        let mut number: T = T::default();
        loop {
            let next = self.next()?;
            if next == delimiter as _ {
                break;
            }
            number = number * T::from(10) + T::from(next - b'0');
        }
        Some(number)
    }

    pub fn parse_name(&mut self, delimiter: char) -> Option<&'a str> {
        let ptr = self.ptr;
        self.skip_to(delimiter as _)?;
        let slice =
            unsafe { std::slice::from_raw_parts(ptr, self.ptr.offset_from(ptr) as usize - 1) };
        Some(std::str::from_utf8(slice).expect("invalid file"))
    }

    pub fn parse_value(&mut self) -> Option<LogStr<'a>> {
        let ch = self.peek()?;
        Some(match ch {
            b'"' => self.parse_str_quote('"')?,
            b'\'' => self.parse_str_quote('\'')?,
            _ => LogStr::new(self.parse_str()?, 0u8 as _),
        })
    }

    pub fn parse_str(&mut self) -> Option<&'a [u8]> {
        let ptr = self.ptr;
        self.skip_to2(b',', b'\r')?;
        let slice =
            unsafe { std::slice::from_raw_parts(ptr, self.ptr.offset_from(ptr) as usize - 1) };
        Some(slice)
    }

    pub fn parse_str_quote(&mut self, quote: char) -> Option<LogStr<'a>> {
        self.skip(1)?;
        let ptr = self.ptr;
        let mut need_replace_quotes = false;

        loop {
            self.skip_to(quote as _)?;
            let next = self.next()?;
            if next == b',' || next == b'\r' {
                break;
            } else if next == quote as _ {
                need_replace_quotes = true;
            }
        }

        let s = unsafe { std::slice::from_raw_parts(ptr, self.ptr.offset_from(ptr) as usize - 2) };
        Some(LogStr::new(
            s,
            if need_replace_quotes {
                quote
            } else {
                0u8 as char
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;

    #[test]
    fn test1() {
        fn test() -> Option<()> {
            let buf = b"57:20.886000-123,EXCP,0,process=rphost,OSThread=252,Exception=0874860b-2b41-45e1-bc2b-6e186eb37771\r\n";
            let mut parser = Parser::new(buf);

            let min: u32 = parser.parse_number(':')?;
            let sec: u32 = parser.parse_number('.')?;
            let msec: u32 = parser.parse_number('-')?;
            let duration: u32 = parser.parse_number(',')?;
            let name = parser.parse_name(',')?;
            let level: u32 = parser.parse_number(',')?;

            assert_eq!(min, 57);
            assert_eq!(sec, 20);
            assert_eq!(msec, 886000);
            assert_eq!(duration, 123);
            assert_eq!(name, "EXCP");
            assert_eq!(level, 0);

            let name = parser.parse_name('=')?;
            let value = parser.parse_value()?;
            assert_eq!(name, "process");
            assert_eq!(value.str(), "rphost");

            let name = parser.parse_name('=')?;
            let value = parser.parse_value()?;
            assert_eq!(name, "OSThread");
            assert_eq!(value.str(), "252");

            let name = parser.parse_name('=')?;
            let value = parser.parse_value()?;
            assert_eq!(name, "Exception");
            assert_eq!(value.str(), "0874860b-2b41-45e1-bc2b-6e186eb37771");

            Some(())
        }
        assert_eq!(test(), Some(()));
    }

    #[test]
    fn test2() {
        fn test() -> Option<()> {
            let buf = b"Test1=\"Test2\"\r\n";
            let mut parser = Parser::new(buf);

            let name = parser.parse_name('=')?;
            let value = parser.parse_value()?;
            assert_eq!(name, "Test1");
            assert_eq!(value.str(), "Test2");

            Some(())
        }
        assert_eq!(test(), Some(()));
    }

    #[test]
    fn test3() {
        fn test() -> Option<()> {
            let buf = b"Test1='Test2'\r\n";
            let mut parser = Parser::new(buf);

            let name = parser.parse_name('=')?;
            let value = parser.parse_value()?;
            assert_eq!(name, "Test1");
            assert_eq!(value.str(), "Test2");

            Some(())
        }
        assert_eq!(test(), Some(()));
    }

    #[test]
    fn test4() {
        fn test() -> Option<()> {
            let buf = b"Test3='Test4''Test5'\r\n";
            let mut parser = Parser::new(buf);

            let name = parser.parse_name('=')?;
            let value = parser.parse_value()?;
            assert_eq!(name, "Test3");
            assert_eq!(value.str(), "Test4'Test5");

            Some(())
        }
        assert_eq!(test(), Some(()));
    }

    #[test]
    fn test5() {
        fn test() -> Option<()> {
            let buf = b"Empty1=,Empty2=\r\n";
            let mut parser = Parser::new(buf);

            let name = parser.parse_name('=')?;
            let value = parser.parse_value()?;
            assert_eq!(name, "Empty1");
            assert_eq!(value.str(), "");

            let name = parser.parse_name('=')?;
            let value = parser.parse_value()?;
            assert_eq!(name, "Empty2");
            assert_eq!(value.str(), "");

            Some(())
        }
        assert_eq!(test(), Some(()));
    }
}
