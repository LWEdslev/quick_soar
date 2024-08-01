use std::{error::Error, fs::File, io::Read};
use std::str::FromStr;
use igc_parser::error::IGCError;
use igc_parser::records::task_info::TaskInfo;
use igc_parser::records::util::{Coordinate, Date, Time};
use igc_parser::records::Record;
use regex::Regex;
use igc_parser::records::fix::Fix as ParserFix;

#[derive(Clone)]
pub struct Fix {
    pub timestamp: u32,
    pub latitude: f32, //positive is north
    pub longitude: f32, //positive is east
    pub alt: Option<i16>,
    pub alt_igc: i16,
}

impl Fix {
    pub fn from(rec: &ParserFix) -> Self {
        let (lat, lon) = coordinates_to_decimals(&rec.coordinates);
        let time = &rec.timestamp;
        Self {
            timestamp: time.seconds_since_midnight(),
            latitude: lat,
            longitude: lon,
            alt: rec.gps_alt,
            alt_igc: rec.pressure_alt,
        }
    }
    pub fn to_string(&self) -> String {
        format!("Fix{{time: {}:{}:{}, lat: {}, lon: {}, alt: {}}}",
                self.timestamp / 3600,
                self.timestamp % 3600 / 60,
                self.timestamp % 60,
                self.latitude,
                self.longitude,
                self.alt_igc)
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.alt.is_some()
    }
}

pub struct TurnpointRecord {
    pub latitude: f32,
    pub longitude: f32,
    pub name: Option<String>,
}

impl TurnpointRecord {
    pub(crate) fn from_c_record_tp(rec: &igc_parser::records::task_info::TaskPoint) -> Self {
        let (latitude, longitude) = coordinates_to_decimals(&rec.coordinate);
        let name = rec.name.clone().map(|s| s.to_string());
        Self {
            latitude,
            longitude,
            name,
        }
    }

    pub fn to_string(&self) -> String {
        format!("Turnpoint{{lat: {}, lon: {}, name: {:?}}}",
                self.latitude, self.longitude, self.name)
    }
}

pub fn get_contents(path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let contents = String::from_utf8_lossy(&buf).to_string();
    Ok(contents)
}

/// Takes contents converts it to `Vec<Record>` and maps it according to `f`
/// # Panics
/// Panics if line in contents is unable to parse
fn map_parsed_contents<F,T>(contents: &str, f: F) -> T
    where F: FnOnce(&Vec<Record>) -> T
{
    let records: Vec<Record> = contents.lines().filter(|line| !line.is_empty()).filter_map(
        |line| {
            match Record::parse(line) {
                Ok(rec) => Some(rec),
                Err(_) => None,
            }
        }
    ).collect();
    f(&records)
}

pub fn get_fixes(contents: &str) -> Vec<Fix> {
    let f = |records: &Vec<Record>| records.iter().filter_map( |record|
        match record {
            Record::B(brecord) => Some(Fix::from(brecord)),
            _ => None,
        }
    ).collect::<Vec<Fix>>();
    map_parsed_contents(contents, f)
}

fn get_l_records_strings(contents: String) -> Vec<String> {
    let f = |records: &Vec<Record>| records.iter().filter_map( |record|
        match record {
            Record::L(lrecord) => Some("L".to_string() + &lrecord.content), //add the L again
            _ => None,
        }
    ).collect::<Vec<String>>();
    map_parsed_contents(&contents, f)
}

pub fn get_turnpoint_locations(contents: &str) -> Vec<TurnpointRecord> {

    fn get_turnpoints_from_l_records(tp_strings: Vec<String>) -> Vec<TurnpointRecord>{
        tp_strings
            .iter()
            .filter_map(|tp| match Record::parse(tp) {
                Ok(record) => match record {
                    Record::C(TaskInfo::TaskPoint(c)) => Some(TurnpointRecord::from_c_record_tp(&c)),
                    _ => None,
                },
                Err(_) => None,
            })
            .filter(|tp| !(tp.longitude == 0. && tp.latitude == 0. && tp.name.is_none())) //removes first and last marker
            .collect()
    }

    let c_record_candidate = get_l_records_strings(contents.to_string())
        .into_iter()
        .filter(|s| s.starts_with("LCU::C"))
        .map(|s| s.replacen("LCU::", "", 1))
        .collect::<Vec<String>>();
    get_turnpoints_from_l_records(c_record_candidate)
}

pub fn get_task_time(contents: &str) -> Option<Time> {
    let f = |records: &Vec<Record>| records.iter().filter_map( |record|
        match record {
            Record::L(lrecord) =>
                if lrecord.content.starts_with("SEEYOU TSK") {
                    Some("L".to_string() + &lrecord.content)
                } else {
                    None
                },
            _ => None,
        }).collect::<Vec<String>>();
    let task_string = map_parsed_contents(contents, f);

    match task_string.first() {
        Some(s) => {
            let regex = Regex::new("TaskTime=[0-9][0-9]:[0-9][0-9]:[0-9][0-9]").ok()?;
            let matc = regex.find(s);

            match matc {
                None => None,
                Some(matc) => {
                    let time_string = &s[matc.start()+"TaskTime=".len() .. matc.end()].to_string();
                    Some(
                        Time::parse(&time_string.replacen(':', "", 3)).ok()?
                    )
                },
            }
        },
        None => None,
    }
}

pub fn get_turnpoint_descriptions(contents: &str) -> Vec<String> {
    let f = |records: &Vec<Record>| records.iter().filter_map( |record|
        match record {
            Record::L(lrecord) =>
                if lrecord.content.starts_with("SEEYOU OZ=") {
                    Some("L".to_string() + &lrecord.content)
                } else {
                    None
                },
            _ => None,
        }).collect::<Vec<String>>();
    map_parsed_contents(contents, f)
}

pub fn get_date(contents: &str) -> Result<Date, IGCError> {
    //If just people used the correct formatting this would be simple!!!
    let hfdte_rec = contents.lines().find(|line| line.starts_with("HFDTE")).unwrap_or("HFDTE999999");
    let first_number = match hfdte_rec.chars().position(|c| c.is_numeric()) {
        Some(i) => i,
        None => return Err(IGCError::DateInitError("No numbers in HFDTE".to_string())),
    };
    Date::parse(&hfdte_rec[first_number..first_number + 6])
}

type Lat = f32;
type Lon = f32;
fn coordinates_to_decimals(rp: &Coordinate) -> (Lat, Lon) {
    let lon = &rp.longitude;
    let lat = &rp.latitude;
    let (lat, lon) = (lat.degrees as f32 + (lat.minutes as f32) / 60.,
                      lon.degrees as f32 + (lon.minutes as f32) / 60.);
    let lat = match rp.latitude.is_north {
        true => lat,
        false => -1. * lat,
    };
    let lon = match rp.longitude.is_east {
        true => lon,
        false => -1. * lon,
    };
    (lat, lon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_should_be_parsed_correctly() {
        if let Ok(Record::B(brecord)) = Record::parse("B0941425152178N00032755WA001130014900854107587076372190033802770100") {
            let fix = Fix::from(&brecord);
            assert_eq!(fix.alt, brecord.gps_alt);
            assert_eq!(fix.latitude, 51.869633);
            assert_eq!(fix.longitude, -0.5459167);
            assert_eq!(fix.timestamp, Time::from_hms(9, 41, 42).unwrap().seconds_since_midnight());
        } else {
            assert!(false)
        };
    }

    #[test]
    fn getting_time() {
        if let Some(time) = get_task_time("LSEEYOU TSK,NoStart=12:57:00,TaskTime=02:00:00,WpDis=False") {
            assert_eq!(time, Time::from_hms(2, 0, 0).unwrap())
        } else {
            assert!(false)
        }
    }

    #[test]
    fn no_time_from_ast() {
        assert_eq!(
            get_task_time(get_contents("examples/ast.igc").expect("file is moved or changed").as_str()),
            None
        )
    }
}