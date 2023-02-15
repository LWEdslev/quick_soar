use std::{error, fs::File, io::Read, string::ParseError};
use std::fmt::Debug;
use igc::records::Record;


fn map_parsed_contents<F,T>(path: &str, function: F) -> Result<Vec<T>, Box<dyn error::Error>>
    where F: FnOnce(&Vec<Record>) -> Vec<T>
{
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let records: Vec<Record> = contents.lines().map(
        |line| {
            Record::parse_line(line).unwrap_or_else(|e| panic!("unable to parse line: {line}"))
        }
    ).collect();
    Ok(function(&records))
}

pub fn get_altitudes(path: &str) -> Vec<i16> {
    let f = |records: &Vec<Record>| records.iter().map(|record| {
        match record {
            Record::B(BRecord) => Some(BRecord.gps_alt),
            _ => None
        }
    }).collect();
    let alts: Vec<Option<i16>> = map_parsed_contents(path, f).unwrap();
    alts.into_iter().flatten().collect()
}