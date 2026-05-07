use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufReader, BufWriter, ErrorKind},
    path::Path,
};

pub(super) const DATA_DIR_FOLDER: &str = "dpm-data";

pub fn read_data_in_path<D, P>(path: P) -> io::Result<D>
where
    D: for<'de> Deserialize<'de>,
    P: AsRef<Path>,
{
    puffin::profile_function!();
    File::open(path).map(BufReader::new).and_then(|reader| {
        pot::from_reader(reader).map_err(|error| io::Error::new(ErrorKind::InvalidData, error))
    })
}

pub fn save_data_to_path<D, P>(path: P, data: &D) -> io::Result<()>
where
    D: Serialize,
    P: AsRef<Path>,
{
    puffin::profile_function!();
    File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map(BufWriter::new)
        .and_then(|writer| {
            pot::to_writer(data, writer)
                .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))
        })
}
