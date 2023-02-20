use crate::parser::task::Turnpoint;
use crate::parser::util::Fix;

type Meters = f32;
type Degrees = f32;

impl Fix {
    fn distance_to(&self, fix: &Fix) -> Meters {
        let from = (self.latitude, self.longitude);
        let to = (fix.latitude, fix.longitude);
        distance_between(from, to)
    }

    fn is_inside(&self, turnpoint: &Turnpoint) -> bool {
        turnpoint.is_inside(self)
    }

    fn bearing_to(&self, fix: &Fix) -> Degrees {
        let delta_lat = self.latitude - fix.latitude;
        let delta_lon = self.longitude - fix.longitude;
        let radians_to_degrees = 57.2957795f32;
        let out = (delta_lat / ( delta_lon * delta_lon + delta_lat * delta_lat).sqrt()).acos()
            * radians_to_degrees * (if delta_lon < 0. {-1.} else {1.}) + (if delta_lon < 0. {360.} else {0.});
        out
    }
}

impl Turnpoint {
    fn is_inside(&self, fix: &Fix) -> bool {
        let from = (self.latitude, self.longitude);
        let to = (fix.latitude, fix.longitude);
        distance_between(from, to) <= self.r1 as f32 //simple beer can model, should be reworked later
    }
}

type Lat = f32;
type Lon = f32;

fn distance_between(from: (Lat, Lon), to: (Lat, Lon)) -> Meters {
    let degrees_to_radians = 0.0174532925f32;
    let (lat1, lon1) = (from.0 * degrees_to_radians, from.1 * degrees_to_radians);
    let (lat2, lon2) = (to.0 * degrees_to_radians, to.1 * degrees_to_radians);

    let x = (lon2 - lon1) * ((lat1 + lat2) / 2.).cos();
    let y = lat2 - lat1;

    (y*y + x*x).sqrt() * 6371_000.
}

#[cfg(test)]

mod tests {
    use igc::records::{BRecord, CRecordTurnpoint, Record};
    use igc::util::Time;
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
        let fix2 = Fix::from(&BRecord::parse("B1246305459371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
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
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02503583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305429371N02503583EA00040000830070040000000010335-005002460010000100").unwrap());
        assert_eq!(fix1.bearing_to(&fix2).round(), 0.);
    }

    #[test]
    fn bearing_nan() {
        let fix1 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        let fix2 = Fix::from(&BRecord::parse("B1246305439371N02403583EA00040000830070040000000010335-005002460010000100").unwrap());
        assert!(fix1.bearing_to(&fix2).is_nan());
    }
}