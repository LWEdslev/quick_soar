use crate::parser::util::Fix;
use crate::{analysis, parser};

pub struct Flight {
    segments: Vec<Segment>
}

impl Flight {
    pub fn make(fixes: Vec<Fix>) -> Self {
        let mut segments: Vec<Segment> = vec![];
        let start_alt = fixes.get(0).unwrap().alt;
        let fixes = fixes.into_iter().filter(|f| f.alt > start_alt + 100).collect::<Vec<Fix>>();

        const DEGREE_BOUNDARY: f32 = 140.;  //turn this many degrees in
        const TIME_WINDOW: u32 = 20;        //this much time
        let target = DEGREE_BOUNDARY / TIME_WINDOW as f32;

        let mut prev_fix = fixes.get(0).unwrap();
        let mut curr_fix = fixes.get(0).unwrap();
        let mut next_fix = fixes.get(0).unwrap();
        let bearing_changes = fixes.iter().map(|f| {
            prev_fix = curr_fix;
            curr_fix = next_fix;
            next_fix = f;
            analysis::util::bearing_change(prev_fix, curr_fix, next_fix)
        }).collect::<Vec<f32>>();

        let mut buildup: Vec<Fix> = vec![];
        let mut buildup_is_glide = true;
        let mut time_buildup = 0;
        let mut short_buildup: Vec<f32> = vec![];
        let mut prev_time = fixes.first().unwrap().timestamp;

        for (fix, change) in fixes.into_iter().zip(bearing_changes) {
            let delta_time = fix.timestamp - prev_time;
            prev_time = fix.timestamp;
            time_buildup += delta_time;
            short_buildup.push(change);
            buildup.push(fix);
            let total_degree_change = short_buildup.iter().sum::<f32>();
            if (total_degree_change / (time_buildup as f32)).abs() >= target {
                //We are turning!
                match buildup_is_glide {
                    true => { //We have just started turning!
                        buildup_is_glide = false;
                        segments.push(Segment::Glide(buildup.clone()));
                        buildup.clear();
                    },
                    false => {}, //We are still turning so wait
                }
            } else {
                //We are going straight!
                match buildup_is_glide {
                    true => {}, //We are still going straight!
                    false => { //We just stopped turning!
                        buildup_is_glide = true;
                        segments.push(Segment::Thermal(buildup.clone()));
                        buildup.clear();
                    },
                }
            }

            while time_buildup >= TIME_WINDOW {
                short_buildup.remove(0);
                time_buildup -= delta_time;
            }

        }

        Self {
            segments,
        }
    }

    pub fn thermal_percentage(&self) -> f32 {
        let thermal_length: f32 =
            self.segments.iter().map(
                |s| match s {
                    Segment::Glide(_) => 0.,
                    Segment::Thermal(v) => v.len() as f32,
                }
            ).sum::<f32>();
        let total_length: f32 =
            self.segments.iter().map(
                |s| match s {
                    Segment::Glide(v) => v.len() as f32,
                    Segment::Thermal(v) => v.len() as f32,
                }
            ).sum::<f32>();
        (thermal_length / total_length) * 100.
    }
}

enum Segment {
    Glide(Vec<Fix>),
    Thermal(Vec<Fix>),
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn segmenting_1_sec() {
        let contents = parser::util::get_contents("examples/aat.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let flight = Flight::make(fixes);
        let target_percentage: f32 = 36.6;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("Segments: {}", &flight.segments.len());
        println!("Percentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    //TODO: Make more tests for segmenting, the entire program relies on this algorithm so it must be bulletproof
}

