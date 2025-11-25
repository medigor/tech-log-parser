use std::{error::Error, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let Some(file_name) = std::env::args().nth(1) else {
        println!("usage: simple-bench /path/to/file/*.log");
        return Ok(());
    };

    let mut count: usize = 0;
    let mut max_properies = 0;

    let start = Instant::now();
    tech_log_parser::parse_file_with_worker(file_name, &mut |event| {
        count += 1;
        max_properies = max_properies.max(event.properties.len());
        Ok(true)
    })?;

    println!("Duration: {:?}", start.elapsed());
    println!("count: {count}");
    println!("max_properies: {max_properies}");

    Ok(())
}
