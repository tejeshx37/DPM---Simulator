use cgal::{num::Rational, RationalPoint};
use csv::{ReaderBuilder, StringRecord};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

#[derive(Debug)]
pub struct Data {
    path: PathBuf,
    output_sender: Sender<Output>,
}

impl Data {
    pub fn new(path: PathBuf, output_sender: Sender<Output>) -> Self {
        Self {
            path,
            output_sender,
        }
    }
}

pub type Output = Result<Vec<RationalPoint>, String>;

#[derive(Debug)]
pub struct PolygonVerticesParser {
    _worker: JoinHandle<()>,
    data_sender: Sender<Data>,
}

impl Default for PolygonVerticesParser {
    fn default() -> Self {
        let (data_sender, data_receiver) = mpsc::channel();
        Self {
            _worker: thread::spawn(move || Self::parser_queue(data_receiver)),
            data_sender,
        }
    }
}

impl PolygonVerticesParser {
    fn parse_record(record: StringRecord, idx: usize) -> Result<RationalPoint, String> {
        record
            .get(0)
            .map(str::parse::<Rational>)
            .transpose()?
            .and_then(|x| {
                record
                    .get(1)
                    .map(str::parse)
                    .map(|result| result.map(|y| RationalPoint::new(x, y)))
            })
            .transpose()?
            .ok_or(format!(
                "Invalid point format {record:#?} at line {}",
                idx + 1
            ))
    }

    fn parse_polygon_vertices(path: &Path) -> Result<Vec<RationalPoint>, String> {
        ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .map_err(|err| err.to_string())
            .and_then(|mut reader| {
                reader
                    .records()
                    .enumerate()
                    .map(|(idx, record_result)| {
                        record_result
                            .map_err(|err| err.to_string())
                            .and_then(|record| Self::parse_record(record, idx))
                    })
                    .collect::<Result<_, String>>()
            })
    }

    fn parser_queue(data_receiver: Receiver<Data>) {
        while let Ok(data) = data_receiver.recv() {
            let _ = data
                .output_sender
                .send(Self::parse_polygon_vertices(&data.path));
        }
    }

    pub fn parse(&self, data: Data) {
        self.data_sender
            .send(data)
            .expect("Failed to send data as worker crashed");
    }
}
