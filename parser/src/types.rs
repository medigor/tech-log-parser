use std::{borrow::Cow, time::Duration};

use chrono::NaiveDateTime;
use serde::Serialize;
use smallvec::SmallVec;

#[derive(Serialize)]
pub struct Event<'a> {
    pub date: NaiveDateTime,
    pub duration: Duration,
    pub name: &'a str,
    pub level: u32,
    pub properties: SmallVec<[(&'a str, LogStr<'a>); 32]>,
}

pub struct LogStr<'a> {
    str: &'a [u8],
    replace_char: char,
}

impl<'a> LogStr<'a> {
    pub fn new(str: &'a [u8], replace_char: char) -> LogStr {
        LogStr { str, replace_char }
    }
    pub fn str(&self) -> Cow<'a, str> {
        let str = String::from_utf8_lossy(self.str);
        match self.replace_char {
            '\'' => Cow::Owned(str.replace(r#"''"#, r#"'"#)),
            '"' => Cow::Owned(str.replace(r#""""#, r#"""#)),
            _ => str,
        }
    }
}

impl<'a> Serialize for LogStr<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.str();
        serializer.serialize_str(&s)?;
        todo!()
    }
}
