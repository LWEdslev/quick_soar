use std::{error::Error, fs::File, io::Read};
use igc::{records::{BRecord, CRecordTurnpoint, Record}, util::{Compass, RawPosition, Time}};


pub struct Fix {
    timestamp: Time,
    latitude: f32, //positive is north
    longitude: f32, //positive is east
    alt: i16,
}

impl Fix {
    fn from(rec: &BRecord) -> Self {
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

pub struct TurnpointLocation {
    latitude: f32,
    longitude: f32,
    name: Option<String>,
}

impl TurnpointLocation {
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
            Record::L(lrecord) => Some(lrecord.log_string.to_string()),
            _ => None,
        }
    ).collect::<Vec<String>>();
    map_parsed_contents(&*contents, f)
}

pub fn get_turnpoints(contents: &str) -> Vec<TurnpointLocation> {

    fn get_turnpoints_from_l_records(tp_strings: Vec<String>) -> Vec<TurnpointLocation>{
        tp_strings
            .iter()
            .filter_map(|tp| match Record::parse_line(tp) {
                Ok(record) => match record {
                    Record::CTurnpoint(c) => Some(TurnpointLocation::from_c_record_tp(&c)),
                    _ => None,
                },
                Err(_) => None,
            })
            .filter(|tp| !(tp.longitude == 0. && tp.latitude == 0. && tp.name == None)) //removes first and last marker
            .collect()
    }

    let c_record_candidate = get_l_records_strings(contents.to_string())
        .into_iter()
        .map(|s| s.replacen("CU::", "", 1))
        .collect::<Vec<String>>();
    get_turnpoints_from_l_records(c_record_candidate)
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
        Compass::East => lon,
        Compass::West => -1. * lon,
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
            assert_eq!(fix.longitude, -0.5459167);
            assert_eq!(fix.timestamp, Time::from_hms(9, 41, 42));
        } else {
            assert!(false)
        };
    }
}