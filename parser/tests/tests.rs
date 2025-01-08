#![cfg(not(miri))]

use std::time::Duration;

use chrono::NaiveDateTime;
use tech_log_parser::Event;

#[allow(dead_code)]
struct OwnEvent {
    date: NaiveDateTime,
    name: String,
    duration: Duration,
    level: u32,
    props: Vec<(String, String)>,
}

impl OwnEvent {
    pub fn get_first_prop(&self, name: &str) -> &str {
        self.props
            .iter()
            .filter(|(x, _)| x == name)
            .next()
            .map(|(_, x)| x.as_str())
            .expect("not found property {name}")
    }
}

impl<'a> From<Event<'a>> for OwnEvent {
    fn from(value: Event<'a>) -> Self {
        OwnEvent {
            name: value.name.to_owned(),
            date: value.date,
            duration: value.duration,
            level: value.level,
            props: value
                .properties
                .iter()
                .map(|(x, y)| (x.to_string(), y.str().to_string()))
                .collect(),
        }
    }
}

#[test]
fn test_parse_file() {
    let mut events = Vec::<OwnEvent>::new();
    println!("{:?}", std::env::current_dir());
    tech_log_parser::parse_file("test-log/24010415.log", &mut |event| {
        events.push(event.into());
        Ok(true)
    })
    .expect("error parsing file");
    assert_eq!(events.len(), 48);
    assert_eq!(events[0].name, "DBV8DBEng");
    assert_eq!(events[1].name, "EXCP");
    assert_eq!(events[1].get_first_prop("process"), "1cv8c");
    assert_eq!(
        events[1].get_first_prop("Exception"),
        "9db1fa37-b455-4f3f-b8dd-7de0ea7d6da3"
    );
    assert!(events[1].get_first_prop("Descr").contains("\r\n"));
    assert!(events[10].get_first_prop("Descr").contains("\r\n"));
    assert!(events[31].get_first_prop("Sql").contains("FROM v8users"));
}

#[test]
fn test_parse_file_with_worker() {
    let mut events = Vec::<OwnEvent>::new();
    println!("{:?}", std::env::current_dir());
    tech_log_parser::parse_file_with_worker("test-log/24010415.log", &mut |event| {
        events.push(event.into());
        Ok(true)
    })
    .expect("error parsing file");
    assert_eq!(events.len(), 48);
}
