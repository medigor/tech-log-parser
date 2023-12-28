use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
    time::Instant,
};

use serde::{ser::SerializeSeq, Serializer};

fn convert_file(
    source: impl AsRef<Path>,
    dest: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&dest.as_ref().parent().ok_or("Invalid destination")?)?;

    let file = File::create(dest)?;
    let mut buf = BufWriter::new(file);
    let mut serializer = serde_json::Serializer::new(&mut buf);
    let mut seq = serializer.serialize_seq(None)?;

    tech_log_parser::parse_file(source, &mut |event| {
        seq.serialize_element(&event)?;
        Ok(())
    })?;

    seq.end()?;
    Ok(())
}

fn file_name_valid(name: impl AsRef<Path>) -> bool {
    name.as_ref()
        .extension()
        .map(|ext| ext == "log")
        .unwrap_or_default()
        && name
            .as_ref()
            .file_stem()
            .and_then(|x| x.to_str())
            .map(|x| x.len() == 8 && x.chars().all(char::is_numeric))
            .unwrap_or_default()
}

fn process_dir(
    source: impl AsRef<Path>,
    dest: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    if source.as_ref().is_dir() {
        for entry in fs::read_dir(source)? {
            let entry = entry?;

            if entry.path().is_dir() {
                let dest = dest.as_ref().join(entry.file_name());
                process_dir(entry.path(), dest)?;
            } else if let Some(name) = entry.path().file_stem().and_then(|x| x.to_str()) {
                let dest = dest.as_ref().join(format!("{name}.json"));
                process_dir(entry.path(), dest)?;
            }
        }
    } else if file_name_valid(&source) {
        let dest = dest
            .as_ref()
            .to_str()
            .ok_or("Invalid destination")?
            .replace(".log", ".json");
        convert_file(source, dest)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let Some(source) = &args.nth(1) else {
        println!("use: converter /path/to/log/dir/source /path/to/log/dir/destination");
        return Ok(());
    };
    let Some(dest) = &args.next() else {
        println!("use: converter /path/to/log/dir/source /path/to/log/dir/destination");
        return Ok(());
    };

    let start = Instant::now();
    process_dir(source, dest)?;
    println!("duration: {:?}", start.elapsed());
    Ok(())
}
