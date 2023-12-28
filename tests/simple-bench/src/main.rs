use std::time::Instant;

use tech_log_parser::parse_file;

fn main() {
    let Some(file_name) = std::env::args().nth(1) else {
        println!("use: bench /path/to/file/*.log");
        return;
    };

    let mut count: usize = 0;
    let mut max_properies = 0;

    let start = Instant::now();
    parse_file(file_name, &mut |event| {
        count += 1;
        max_properies = max_properies.max(event.properties.len());
        Ok(())
    })
    .expect("failed to parse file");

    println!("Duration: {:?}", start.elapsed());
    println!("count: {count}");
    println!("max_properies: {max_properies}");
}
