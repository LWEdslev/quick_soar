use std::{error::Error, fs::File, io::Read};
use igc::{records::Record, util::Time};
use igc::records::BRecord;
use igc::util::Compass;

pub struct Fix {
    timestamp: Time,
    latitude: f32, //positive is north
    longitude: f32, //positive is east
    alt: i16,
}

impl Fix {
    fn from(rec: &BRecord) -> Self {
        let (lat, lon) = (rec.pos.lat.0, rec.pos.lon.0);
        let (lat, lon) = (lat.degrees as f32 + (lat.minute_thousandths as f32 / 1000.) / 60.,
                          lon.degrees as f32 + (lon.minute_thousandths as f32 / 1000.) / 60.);
        let lat = match rec.pos.lat.0.sign {
            Compass::North => lat,
            Compass::South => -1. * lat,
            _ => panic!("latitude was neither north nor south")
        };
        let lon = match rec.pos.lon.0.sign {
            Compass::East => lon,
            Compass::West => -1. * lon,
            _ => panic!("longitude was neither east nor west")
        };
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
        format!("Fix{{time: {}:{}:{}, lon: {}, lat: {}, alt: {}}}",
                self.timestamp.hours,
                self.timestamp.minutes,
                self.timestamp.seconds,
                self.longitude,
                self.latitude,
                self.alt)
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
fn map_parsed_contents<F,T>(contents: String, f: F) -> T
    where F: FnOnce(&Vec<Record>) -> T
{
    let records: Vec<Record> = contents.lines().map(
        |line| {
            Record::parse_line(line).unwrap_or_else(|e| panic!("unable to parse line: {line}, because error: {:?}", e))
        }
    ).collect();
    f(&records)
}

pub fn get_fixes(contents: String) -> Vec<Fix> {
    let f = |records: &Vec<Record>| records.iter().filter_map( |record|
        match record {
            Record::B(brecord) => Some(Fix::from(&brecord)),
            _ => None,
        }
    ).collect::<Vec<Fix>>();
    map_parsed_contents(contents, f)
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