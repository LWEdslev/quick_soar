use igc::util::Time;
use crate::parser::task::Turnpoint;
use crate::parser::util::Fix;

type FloatMeters = f32;
type Degrees = f32;

impl Fix {
    pub(crate) fn distance_to(&self, fix: &Fix) -> FloatMeters {
        let from = (self.latitude, self.longitude);
        let to = (fix.latitude, fix.longitude);
        distance_between(from, to)
    }

    fn is_inside(&self, turnpoint: &Turnpoint) -> bool {
        turnpoint.is_inside(self)
    }



    fn bearing_to(&self, fix: &Fix) -> Degrees {
        let delta_lat = fix.latitude - self.latitude ;
        let delta_lon = fix.longitude - self.longitude;
        let out = (delta_lat / ( delta_lon * delta_lon + delta_lat * delta_lat).sqrt()).acos().to_degrees()
                    * delta_lon.signum() + (if delta_lon.is_sign_negative() {360.} else {0.});
        out
    }
}

impl Turnpoint {
    pub(crate) fn is_inside(&self, fix: &Fix) -> bool {
        let from = (self.latitude, self.longitude);
        let to = (fix.latitude, fix.longitude);
        distance_between(from, to) <= self.r1 as f32 //simple beer can model, should be reworked later
    }

    pub(crate) fn distance_to(&self, turnpoint: &Turnpoint) -> FloatMeters {
        let from = (self.latitude, self.longitude);
        let to = (turnpoint.latitude, turnpoint.longitude);
        distance_between(from, to)
    }
}

type Lat = f32;
type Lon = f32;

fn distance_between(from: (Lat, Lon), to: (Lat, Lon)) -> FloatMeters {
    let (lat1, lon1) = (from.0.to_radians(), from.1.to_radians());
    let (lat2, lon2) = (to.0.to_radians(), to.1.to_radians());

    let x = (lon2 - lon1) * ((lat1 + lat2) / 2.).cos();
    let y = lat2 - lat1;

    (y*y + x*x).sqrt() * 6371_000.
}

/// Negative is clockwise.
/// Positive is counter-clockwise.
pub fn bearing_change(first: &Fix, second: &Fix, last: &Fix) -> Degrees {
    let bearing1 = first.bearing_to(&second);
    quick_bearing_change(bearing1, second, last)
}

/// Negative is clockwise.
/// Positive is counter-clockwise.
pub fn quick_bearing_change(prev_bearing: Degrees, second: &Fix, last: &Fix) -> Degrees {
    let bearing1 = prev_bearing;
    let bearing2 = second.bearing_to(&last);
    let delta =  bearing1 - bearing2;

    if delta.is_nan() { return 0.; };

    if delta > 180. {
        delta - 360.
    } else if delta < -180. {
        delta + 360.
    } else {
        delta
    }
}

pub trait Offsetable {
    fn offset(&mut self, offset: i8);
}

impl Offsetable for Time {
    fn offset(&mut self, offset: i8) {
        let h = self.hours as i8 + offset;
        self.hours = if h >= 24 { (h - 24) as u8 } else { h as u8 }
    }
}

#[cfg(test)]

mod tests {
    use igc::records::{BRecord, CRecordTurnpoint, Record};
    use igc::util::Time;
    use crate::parser;
    use crate::parser::util::TurnpointRecord;
    use super::*;

    #[test]
    fn fix_to_fix_distance() {
        let brecord1 = BRecord::parse("B1232495601387N00905124EA007720090900600410898098882080009801190100").unwrap();
        let fix1 = Fix::from(&brecord1);

        let brecord2 = BRecord::parse("B1232495601387N00905124EA007720090900600410898098882080009801190100").unwrap();
        let fix2 = Fix::from(&brecord2);

        assert_eq!(fix1.distance_to(&fix2), 0.);

        let brecord2 = BRecord::parse("B1232495602387N00905124EA007720090900600410898098882080009801190100").unwrap();
        let fix2 = Fix::from(&brecord2);

        assert_eq!(fix1.distance_to(&fix2).floor(), 1853.);

        let brecord2 = BRecord::parse("B1232495602387N00906124EA007720090900600410898098882080009801190100").unwrap();
        let fix2 = Fix::from(&brecord2);

        assert_eq!(fix1.distance_to(&fix2).floor(), 2122.);
    }

    #[test]
    fn is_inside_turnpoint() {
        let fix = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let turnpoint = Turnpoint::parse(
            "SEEYOU OZ=2,Style=1,SpeedStyle=1,R1=3000m,A1=180,R2=0m,A2=0,MaxAlt=0.0m,AAT=1",
            TurnpointRecord::from_c_record_tp(
                &CRecordTurnpoint::parse("C5439183N02403467E220Pociunai").unwrap()
            ));
    }

    #[test]
    fn bearing_90() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305439371N02503583EA00040000830070040000000010335-005002460010000100").unwrap());
        assert_eq!(fix1.bearing_to(&fix2).round(), 90.);
    }

    #[test]
    fn bearing_180() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305429371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        assert_eq!(fix1.bearing_to(&fix2).round(), 180.);
    }

    #[test]
    fn bearing_270() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305439371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());
        assert_eq!(fix1.bearing_to(&fix2).round(), 270.);
    }

    #[test]
    fn bearing_0() {
        let fix1 = Fix::from(&BRecord::parse("B1246305429371N02503583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305439371N02503583EA00040000830070040000000010335-005002460010000100").unwrap());
        assert_eq!(fix1.bearing_to(&fix2).round(), 0.);
    }

    #[test]
    fn bearing_nan() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        assert!(fix1.bearing_to(&fix2).is_nan());
    }

    #[test]
    fn three_bearing_test_90() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305439371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix3 = Fix::from(&BRecord::parse("B1246305339371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());

        assert_eq!(bearing_change(&fix1, &fix2, &fix3).round(), 90.)
    }

    #[test]
    fn three_bearing_test_minus_90() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305439371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix3 = Fix::from(&BRecord::parse("B1246305339371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());

        assert_eq!(bearing_change(&fix3, &fix2, &fix1).round(), -90.)
    }

    #[test]
    fn three_bearing_test_0() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305439371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix3 = Fix::from(&BRecord::parse("B1246305339371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());

        assert_eq!(bearing_change(&fix1, &fix1, &fix1).round(), 0.)
    }

    #[test]
    fn three_bearing_test_over_0_boundary() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305539371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix3 = Fix::from(&BRecord::parse("B1246305639371N02303583EA00040000830070040000000010335-005002460010000100").unwrap());

        assert_eq!(bearing_change(&fix1, &fix2, &fix3).round(), 90.)
    }

    #[test]
    fn positive_timezone() {
        let contents = parser::util::get_contents("examples/ast.igc").unwrap();
        let pilot_info = parser::pilot_info::PilotInfo::parse(&contents);
        let mut time = Time::from_hms(10, 43, 56);
        time.offset(pilot_info.time_zone);
        assert_eq!(pilot_info.time_zone, 2);
        assert_eq!(time, Time::from_hms(12, 43, 56));
    }

    #[test]
    fn negative_timezone() {

    }
}