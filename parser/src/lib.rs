use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
    time::Duration,
};

use chrono::{NaiveDate, NaiveDateTime, Timelike};
use parser::Parser;
use smallvec::SmallVec;
use types::Event;

mod parser;
mod types;

pub fn parse_record<'a>(parser: &'a mut Parser, date: NaiveDateTime) -> Option<Event<'a>> {
    let min = parser.parse_u32(':')?;
    let sec = parser.parse_u32('.')?;
    let msec = parser.parse_u32('-')?;
    let duration = parser.parse_u64(',')?;
    let name = parser.parse_name(',')?;
    let level = parser.parse_u32(',')?;

    let mut properties = SmallVec::new();

    loop {
        let name = parser.parse_name('=')?;
        let value = parser.parse_value()?;
        properties.push((name, value));

        if parser.peek()? == b'\n' {
            parser.skip(1)?;
            break;
        }
    }

    let date = date
        .with_minute(min)
        .and_then(|date| date.with_minute(min))
        .and_then(|date| date.with_second(sec))
        .and_then(|date| date.with_nanosecond(msec * 1000))
        .expect("failed to parse date");

    Some(Event {
        date,
        duration: Duration::from_micros(duration),
        name,
        level,
        properties,
    })
}

pub fn parse_buffer<'a, F>(buffer: &'a [u8], date: NaiveDateTime, action: &'a mut F) -> usize
where
    F: FnMut(Event),
{
    let mut parser = Parser::new(buffer);
    loop {
        let position = parser.position();
        match parse_record(&mut parser, date) {
            Some(event) => action(event),
            None => return position,
        }
    }
}

fn parse_date_file(file_name: impl AsRef<Path>) -> Option<NaiveDateTime> {
    let name = Path::new(file_name.as_ref()).file_name()?.to_str()?;
    let year: i32 = name[..2].parse().ok()?;
    let month: u32 = name[2..4].parse().ok()?;
    let day: u32 = name[4..6].parse().ok()?;
    let hour: u32 = name[6..8].parse().ok()?;

    let date = NaiveDate::from_ymd_opt(2000 + year, month, day)?.and_hms_opt(hour, 0, 0)?;

    Some(date)
}

pub fn parse_file<F, P>(file_name: P, action: &mut F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(Event),
    P: AsRef<Path>,
{
    let date = parse_date_file(&file_name).ok_or("invalid file name")?;

    let mut reader = File::open(&file_name)?;
    reader.seek(std::io::SeekFrom::Start(3))?;

    let mut buffer = Vec::<u8>::with_capacity(1024 * 1024);
    buffer.extend((0..1024 * 1024).map(|_| 0));
    let mut offset = 0usize;

    loop {
        let len = reader.read(&mut buffer[offset..])?;
        if len == 0 {
            break;
        }
        let len = len + offset;

        let read = parse_buffer(&buffer[0..len], date, action);

        if read == 0 {
            buffer.extend((0..1024 * 1024).map(|_| 0));
        }

        for i in read..len {
            buffer[i - read] = buffer[i];
        }
        offset = len - read;
    }

    Ok(())
}
