use crate::parser::util::Fix;

type Meters = f32;

impl Fix {
    fn distance_to(&self, fix: Fix) -> Meters {
        let from = (self.latitude, self.longitude);
        let to = (fix.latitude, fix.longitude);
        distance_between(from, to)
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

    (y*y + x*x).sqrt() * (6371_000.)//nautical_miles_to_meters
}

#[cfg(test)]

mod tests {
    use igc::records::{BRecord, Record};
    use igc::util::Time;
    use super::*;

    #[test]
    fn fix_to_fix_distance() {
        /*
        B1232495601387N00905124EA007720090900600410898098882080009801190100
B1232535601354N00905042EA007770091500600410341096782580009401180120
         */
        let brecord1 = BRecord::parse("B1232495601387N00905124EA007720090900600410898098882080009801190100").unwrap();
        let fix1 = Fix::from(&brecord1);

        let brecord2 = BRecord::parse("B1232495601387N00905124EA007720090900600410898098882080009801190100").unwrap();
        let fix2 = Fix::from(&brecord2);

        assert_eq!(fix1.distance_to(fix2), 0.);

        let brecord2 = BRecord::parse("B1232495602387N00905124EA007720090900600410898098882080009801190100").unwrap();
        let fix2 = Fix::from(&brecord2);

        assert_eq!(fix1.distance_to(fix2).floor(), 1853.);

        let brecord2 = BRecord::parse("B1232495602387N00906124EA007720090900600410898098882080009801190100").unwrap();
        let fix2 = Fix::from(&brecord2);

        assert_eq!(fix1.distance_to(fix2).floor(), 2122.);
    }
}