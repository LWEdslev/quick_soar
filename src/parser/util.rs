use std::{error::Error, fs::File, io::Read};
use std::str::FromStr;
use igc::{records::{BRecord, CRecordTurnpoint, Record}, util::{Compass, RawPosition, Time}};
use regex::{Match, Regex};


pub struct Fix {
    pub timestamp: Time,
    pub latitude: f32, //positive is north
    pub longitude: f32, //positive is west
    pub alt: i16,
}

impl Fix {
    pub fn from(rec: &BRecord) -> Self {
        let (lat, lon) = raw_position_to_decimals(&rec.pos);
        let time = &rec.timestamp;
        let (h,m,s) = (time.hours, time.minutes, time.seconds);
        Self {
            timestamp: Time::from_hms(h,m,s),
            latitude: lat,
            longitude: lon,
            alt: rec.gps_alt
        }
    }
    pub fn to_string(&self) -> String {
        format!("Fix{{time: {}:{}:{}, lat: {}, lon: {}, alt: {}}}",
                self.timestamp.hours,
                self.timestamp.minutes,
                self.timestamp.seconds,
                self.latitude,
                self.longitude,
                self.alt)
    }
}

pub struct TurnpointRecord {
    pub latitude: f32,
    pub longitude: f32,
    pub name: Option<String>,
}

impl TurnpointRecord {
    pub(crate) fn from_c_record_tp(rec: &CRecordTurnpoint) -> Self {
        let (latitude, longitude) = raw_position_to_decimals(&rec.position);
        let name = match rec.turnpoint_name {
            Some(s) => Some(s.to_string()),
            None => None,
        };
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
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Takes contents converts it to `Vec<Record>` and maps it according to `f`
/// # Panics
/// Panics if line in contents is unable to parse
fn map_parsed_contents<F,T>(contents: &str, f: F) -> T
    where F: FnOnce(&Vec<Record>) -> T
{
    let records: Vec<Record> = contents.lines().map(
        |line| {
            Record::parse_line(line).unwrap_or_else(|e| panic!("unable to parse line: {line}, because error: {:?}", e))
        }
    ).collect();
    f(&records)
}

pub fn get_fixes(contents: &str) -> Vec<Fix> {
    let f = |records: &Vec<Record>| records.iter().filter_map( |record|
        match record {
            Record::B(brecord) => Some(Fix::from(&brecord)),
            _ => None,
        }
    ).collect::<Vec<Fix>>();
    map_parsed_contents(contents, f)
}

fn get_l_records_strings(contents: String) -> Vec<String> {
    let f = |records: &Vec<Record>| records.iter().filter_map( |record|
        match record {
            Record::L(lrecord) => Some("L".to_string() + lrecord.log_string), //add the L again
            _ => None,
        }
    ).collect::<Vec<String>>();
    map_parsed_contents(&*contents, f)
}

pub fn get_turnpoint_locations(contents: &str) -> Vec<TurnpointRecord> {

    fn get_turnpoints_from_l_records(tp_strings: Vec<String>) -> Vec<TurnpointRecord>{
        tp_strings
            .iter()
            .filter_map(|tp| match Record::parse_line(tp) {
                Ok(record) => match record {
                    Record::CTurnpoint(c) => Some(TurnpointRecord::from_c_record_tp(&c)),
                    _ => None,
                },
                Err(_) => None,
            })
            .filter(|tp| !(tp.longitude == 0. && tp.latitude == 0. && tp.name == None)) //removes first and last marker
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
                if lrecord.log_string.starts_with("SEEYOU TSK") {
                    Some("L".to_string() + lrecord.log_string)
                } else {
                    None
                },
            _ => None,
        }).collect::<Vec<String>>();
    let task_string = map_parsed_contents(&*contents, f);
    assert_eq!(task_string.len(), 1);

    match task_string.first() {
        Some(s) => {
            let regex = Regex::new("TaskTime=[0-9][0-9]:[0-9][0-9]:[0-9][0-9]").unwrap();
            let matc = regex.find(s);

            match matc {
                None => None,
                Some(matc) => {
                    let time_string = &s[matc.start()+"TaskTime=".len() .. matc.end()].to_string();
                    Some(
                        Time::from_str(&*time_string.replacen(":", "", 3)).unwrap()
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
                if lrecord.log_string.starts_with("SEEYOU OZ=") {
                    Some("L".to_string() + lrecord.log_string)
                } else {
                    None
                },
            _ => None,
        }).collect::<Vec<String>>();
    map_parsed_contents(contents, f)
}

type Lat = f32;
type Lon = f32;
fn raw_position_to_decimals(rp: &RawPosition) -> (Lat, Lon) {
    let lon = rp.lon.0;
    let lat = rp.lat.0;
    let (lat, lon) = (lat.degrees as f32 + (lat.minute_thousandths as f32 / 1000.) / 60.,
                      lon.degrees as f32 + (lon.minute_thousandths as f32 / 1000.) / 60.);
    let lat = match rp.lat.0.sign {
        Compass::North => lat,
        Compass::South => -1. * lat,
        _ => panic!("latitude was neither north nor south")
    };
    let lon = match rp.lon.0.sign {
        Compass::East => -1. * lon,
        Compass::West => lon,
        _ => panic!("longitude was neither east nor west")
    };
    (lat, lon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fix_should_be_parsed_correctly() {
        if let Ok(Record::B(brecord)) = Record::parse_line("B0941425152178N00032755WA001130014900854107587076372190033802770100") {
            let fix = Fix::from(&brecord);
            assert_eq!(fix.alt, brecord.gps_alt);
            assert_eq!(fix.latitude, 51.869633);
            assert_eq!(fix.longitude, 0.5459167);
            assert_eq!(fix.timestamp, Time::from_hms(9, 41, 42));
        } else {
            assert!(false)
        };
    }

    #[test]
    fn getting_time() {
        if let Some(time) = get_task_time("LSEEYOU TSK,NoStart=12:57:00,TaskTime=02:00:00,WpDis=False") {
            assert_eq!(time, Time::from_hms(2, 0, 0))
        } else {
            assert!(false)
        }
    }

    #[test]
    fn no_time_from_ast() {
        assert_eq!(
            get_task_time(get_contents("examples/ast.igc").unwrap().as_str()),
            None
        )
    }
}