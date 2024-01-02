use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Deserialize)]
pub enum StrFilter {
    Equal(String),
    Contains(String),
    InList(Vec<String>),
}

impl StrFilter {
    pub fn check(&self, value: &str) -> bool {
        match self {
            StrFilter::Equal(s) => s == value,
            StrFilter::Contains(s) => value.contains(s),
            StrFilter::InList(s) => s.iter().any(|x| x == value),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PropFilter {
    pub name: String,
    pub filter: StrFilter,
}

impl PropFilter {
    pub fn check(&self, props: &[(&str, tech_log_parser::LogStr<'_>)]) -> bool {
        for (name, value) in props {
            if name == &self.name && self.filter.check(&value.str()) {
                return true;
            }
        }
        false
    }
}

#[derive(Deserialize)]
pub enum DataFilter {
    GreaterOrEqual(NaiveDateTime),
    LessOrEqual(NaiveDateTime),
}

impl DataFilter {
    pub fn check(&self, value: &NaiveDateTime) -> bool {
        match self {
            DataFilter::GreaterOrEqual(date) => value >= date,
            DataFilter::LessOrEqual(date) => value <= date,
        }
    }
}

#[derive(Deserialize)]
pub enum DurationFilter {
    GreaterOrEqual(u128),
    LessOrEqual(u128),
}

impl DurationFilter {
    pub fn check(&self, value: &u128) -> bool {
        match self {
            DurationFilter::GreaterOrEqual(dur) => value >= dur,
            DurationFilter::LessOrEqual(dur) => value <= dur,
        }
    }
}

#[derive(Deserialize)]
pub enum Filter {
    Date(DataFilter),
    Duration(DurationFilter),
    Name(StrFilter),
    Prop(PropFilter),
}

impl Filter {
    pub fn check(&self, event: &tech_log_parser::Event) -> bool {
        match self {
            Filter::Date(filter) => filter.check(&event.date),
            Filter::Duration(filter) => filter.check(&event.duration.as_micros()),
            Filter::Name(filter) => filter.check(event.name),
            Filter::Prop(filter) => filter.check(event.properties.as_slice()),
        }
    }
}
