use std::{borrow::Cow, time::Duration};

use chrono::NaiveDateTime;
use serde::{Serialize, Serializer, ser::SerializeStruct};
use smallvec::SmallVec;

pub struct Event<'a> {
    pub date: NaiveDateTime,
    pub duration: Duration,
    pub name: &'a str,
    pub level: u32,
    pub properties: SmallVec<[(&'a str, LogStr<'a>); 32]>,
}

impl<'a> Serialize for Event<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sstruct = serializer.serialize_struct("Event", 5)?;

        sstruct.serialize_field("Date", &self.date)?;
        sstruct.serialize_field("Duration", &self.duration.as_micros())?;
        sstruct.serialize_field("Name", &self.name)?;
        sstruct.serialize_field("Level", &self.level)?;
        sstruct.serialize_field("Props", &self.properties)?;

        sstruct.end()
    }
}

pub struct LogStr<'a> {
    str: &'a [u8],
    replace_char: char,
}

impl<'a> LogStr<'a> {
    pub fn new(str: &'a [u8], replace_char: char) -> LogStr<'a> {
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
        serializer.serialize_str(&s)
    }
}
