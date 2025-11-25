use std::{
    fs::File,
    io::{ErrorKind, Read, Seek},
    path::Path,
};

pub(crate) fn open_file<P>(file_name: P) -> std::io::Result<File>
where
    P: AsRef<Path>,
{
    let mut file = File::open(&file_name)?;
    let mut bom = [0u8; 3];
    match file.read_exact(&mut bom) {
        Ok(_) => (),
        Err(err) => {
            if err.kind() == ErrorKind::UnexpectedEof {
                return Ok(file);
            } else {
                return Err(err);
            }
        }
    };
    if bom != [0xEF, 0xBB, 0xBF] {
        file.seek(std::io::SeekFrom::Start(0))?;
    };
    Ok(file)
}
