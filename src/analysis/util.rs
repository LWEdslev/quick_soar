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

    pub(crate) fn distance_to_tp(&self, turnpoint: &Turnpoint) -> FloatMeters {
        let from = (self.latitude, self.longitude);
        let to = (turnpoint.latitude, turnpoint.longitude);
        distance_between(from, to)
    }

    fn bearing_to(&self, fix: &Fix) -> Degrees {
        let delta_lat = fix.latitude - self.latitude ;
        let delta_lon = fix.longitude - self.longitude;
        
        (delta_lat / ( delta_lon * delta_lon + delta_lat * delta_lat).sqrt()).acos().to_degrees()
                    * delta_lon.signum() + (if delta_lon.is_sign_negative() {360.} else {0.})
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

    (y*y + x*x).sqrt() * 6_371_000.
}

/// Negative is clockwise.
/// Positive is counter-clockwise.
pub fn bearing_change(first: &Fix, second: &Fix, last: &Fix) -> Degrees {
    let bearing1 = first.bearing_to(second);
    quick_bearing_change(bearing1, second, last)
}

/// Negative is clockwise.
/// Positive is counter-clockwise.
pub fn quick_bearing_change(prev_bearing: Degrees, second: &Fix, last: &Fix) -> Degrees {
    let bearing1 = prev_bearing;
    let bearing2 = second.bearing_to(last);
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
