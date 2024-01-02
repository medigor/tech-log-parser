use std::error::Error;

use addin1c::{name, AddinResult, MethodInfo, PropInfo, SimpleAddin, Variant};

use crate::filters;

pub struct Parser {
    last_error: Option<Box<dyn Error>>,
}

impl Parser {
    pub fn new() -> Self {
        Self { last_error: None }
    }

    fn last_error(&mut self, value: &mut Variant) -> AddinResult {
        match &self.last_error {
            Some(err) => value
                .set_str1c(err.to_string().as_str())
                .map_err(|e| e.into()),
            None => value.set_str1c("").map_err(|e| e.into()),
        }
    }

    fn parse_file(
        &mut self,
        file_name: &mut Variant,
        filter: &mut Variant,
        limit: &mut Variant,
        ret_value: &mut Variant,
    ) -> AddinResult {
        use serde::ser::SerializeSeq;
        use serde::Serializer;

        let file_name = file_name.get_string()?;
        let filter = filter.get_blob()?;
        let limit = limit.get_i32()?;

        let filter: Vec<filters::Filter> = serde_json::from_slice(filter)?;

        let mut buf = Vec::<u8>::new();
        let mut serializer = serde_json::Serializer::new(&mut buf);
        let mut seq = serializer.serialize_seq(None)?;

        let mut count: i32 = 0;

        tech_log_parser::parse_file_with_worker(file_name, &mut |event| {
            for filter in filter.iter() {
                if !filter.check(&event) {
                    return Ok(true);
                }
            }

            seq.serialize_element(&event)?;

            if limit > 0 {
                count += 1;
                if count == limit {
                    return Ok(false);
                }
            }

            Ok(true)
        })?;

        seq.end()?;

        ret_value.set_blob(&buf)?;

        Ok(())
    }
}

impl SimpleAddin for Parser {
    fn name() -> &'static [u16] {
        name!("TechLogParser")
    }

    fn save_error(&mut self, err: Option<Box<dyn Error>>) {
        self.last_error = err;
    }

    fn properties() -> &'static [PropInfo<Self>] {
        &[PropInfo {
            name: name!("LastError"),
            getter: Some(Self::last_error),
            setter: None,
        }]
    }

    fn methods() -> &'static [addin1c::MethodInfo<Self>]
    where
        Self: Sized,
    {
        &[MethodInfo {
            name: name!("ParseFile"),
            method: addin1c::Methods::Method3(Self::parse_file),
        }]
    }
}
