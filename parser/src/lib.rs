use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
    sync::mpsc,
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

pub fn parse_buffer<'a, F>(
    buffer: &'a [u8],
    date: NaiveDateTime,
    action: &'a mut F,
) -> Result<usize, Box<dyn std::error::Error>>
where
    F: FnMut(Event) -> Result<(), Box<dyn std::error::Error>>,
{
    let mut parser = Parser::new(buffer);
    loop {
        let position = parser.position();
        match parse_record(&mut parser, date) {
            Some(event) => action(event)?,
            None => return Ok(position),
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
    F: FnMut(Event) -> Result<(), Box<dyn std::error::Error>>,
    P: AsRef<Path>,
{
    let date = parse_date_file(&file_name).ok_or("invalid file name")?;

    let mut reader = File::open(&file_name)?;
    reader.seek(std::io::SeekFrom::Start(3))?;

    let mut buffer = Vec::<u8>::with_capacity(1024 * 1024);
    buffer.extend((0..buffer.capacity()).map(|_| 0));
    let mut offset = 0usize;

    loop {
        let len = reader.read(&mut buffer[offset..])?;
        if len == 0 {
            break;
        }
        let len = len + offset;

        let read = parse_buffer(&buffer[0..len], date, action)?;

        if read == 0 {
            buffer.extend((0..buffer.len()).map(|_| 0));
        }

        for i in read..len {
            buffer[i - read] = buffer[i];
        }
        offset = len - read;
    }

    Ok(())
}

pub fn parse_file_with_worker<F, P>(
    file_name: P,
    action: &mut F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(Event) -> Result<(), Box<dyn std::error::Error>>,
    P: AsRef<Path>,
{
    let date = parse_date_file(&file_name).ok_or("invalid file name")?;

    let (parser_sender, thread_receiver) = mpsc::channel::<Option<Vec<u8>>>();
    let (thread_sender, parser_receiver) = mpsc::channel::<(usize, Vec<u8>)>();

    for _ in 0..3 {
        let mut buf = Vec::<u8>::with_capacity(1 * 1024 * 1024);
        buf.extend((0..buf.capacity()).map(|_| 0));
        parser_sender.send(Some(buf))?;
    }

    let file_name = file_name.as_ref().to_owned();

    let worker = std::thread::spawn(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut file = File::open(file_name)?;
            file.seek(std::io::SeekFrom::Start(3))?;
            loop {
                let Some(mut buf) = thread_receiver.recv()? else {
                    return Ok(());
                };
                let offset = buf.len() / 2;
                let size = file.read(&mut buf[offset..])?;
                thread_sender.send((size, buf))?;
            }
        },
    );

    let mut rem = Vec::<u8>::new();
    loop {
        let (size, mut buf) = parser_receiver.recv()?;

        if size == 0 {
            parser_sender.send(None)?;
            break;
        }
        let end = buf.len() / 2 + size;
        let start = buf.len() / 2 - rem.len();

        buf[start..start + rem.len()].copy_from_slice(&rem);
        rem.clear();
        let mut read = parse_buffer(&buf[start..end], date, action)?;

        if read == 0 {
            let mut big_buffer = Vec::<u8>::with_capacity(buf.capacity() * 5);
            big_buffer.extend(&buf[start..end]);
            parser_sender.send(Some(buf))?;
            loop {
                let (size, buf) = parser_receiver.recv()?;
                if size == 0 {
                    parser_sender.send(None)?;
                    break;
                }
                big_buffer.extend(&buf[buf.len() / 2..buf.len() / 2 + size]);
                parser_sender.send(Some(buf))?;
                read = parse_buffer(&big_buffer, date, action)?;
                if read > 0 {
                    rem.extend(&big_buffer[read..]);
                    break;
                }
            }
        } else {
            rem.extend(&buf[start + read..end]);
            parser_sender.send(Some(buf))?;
        }

        if worker.is_finished() {
            match worker.join().expect("thread paniced") {
                Ok(_) => (),
                Err(err) => return Err(err),
            }
            break;
        };
    }

    Ok(())
}
