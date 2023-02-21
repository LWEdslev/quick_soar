use igc::util::Time;
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

        const DEGREE_BOUNDARY: f32 = 120.;  //turn this many degrees in
        const TIME_WINDOW: u32 = 15;        //this much time
        const THERMAL_TIME_LIMIT: u32 = 45; //time one has to stop thermalling for it to be a glide
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
                        let time_of_segment = buildup.last().unwrap().timestamp - buildup.first().unwrap().timestamp;
                        let segment = if time_of_segment <=  THERMAL_TIME_LIMIT {
                            let mut prev_segment = match segments.pop() {
                                None => vec![],
                                Some(prev_segment) => match prev_segment {
                                    Segment::Glide(v) => v,
                                    Segment::Thermal(v) => v,
                                }
                            };

                            prev_segment.append(&mut buildup.clone());

                            Segment::Thermal(prev_segment) //We did not stop turning for long enough
                        } else {
                            Segment::Glide(buildup.clone())
                        };
                        segments.push(segment);
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
                        let mut prev_segment = match segments.last() {
                            None => vec![],
                            Some(prev_segment) => {
                                match prev_segment {
                                    Segment::Glide(_) => vec![],
                                    Segment::Thermal(_) => {
                                        if let Segment::Thermal(v) = segments.pop().unwrap() {
                                            v
                                        } else {
                                            panic!("unreachable")
                                        }
                                    }
                                }
                            }
                        };

                        prev_segment.append(&mut buildup.clone());


                        let segment = Segment::Thermal(prev_segment);
                        segments.push(segment);
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

    pub fn print_segments(&self) {
        for segment in &self.segments {
            let (first, last, glide) = match segment {
                Segment::Glide(v) => (v.first().unwrap(), v.last().unwrap(), true),
                Segment::Thermal(v) => (v.first().unwrap(), v.last().unwrap(), false),
            };
            let first_time = Time::from_hms((first.timestamp / 3600) as u8, ((first.timestamp % 3600) / 60) as u8, (first.timestamp % 60) as u8);
            let last_time = Time::from_hms((last.timestamp / 3600) as u8, ((last.timestamp % 3600) / 60) as u8, (last.timestamp % 60) as u8);
            if glide { println!("Glide:") } else { println!("Thermal") }
            println!("\t{}:{}:{} -> {}:{}:{}", first_time.hours, first_time.minutes, first_time.seconds, last_time.hours, last_time.minutes, last_time.seconds);

        }
    }
}

enum Segment {
    Glide(Vec<Fix>),
    Thermal(Vec<Fix>),
}

impl Segment {
    fn total_time(&self) -> u32 {
        let inner = match self {
            Segment::Glide(v) => v,
            Segment::Thermal(v) => v,
        };
        inner.last().unwrap().timestamp - inner.first().unwrap().timestamp
    }
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

