use std::ops::Deref;
use std::rc::Rc;
use igc::util::Time;
use crate::parser::util::Fix;
use crate::{analysis, parser};
use crate::analysis::util::Offsetable;

pub struct Flight {
    pub fixes: Vec<Rc<Fix>>,
    pub segments: Vec<Segment>
}

impl Flight {
    pub fn make(fixes: Vec<Fix>) -> Self {
        let mut segments: Vec<Segment> = vec![];
        let start_alt = fixes.get(0).unwrap().alt;
        let fixes = fixes.into_iter().filter(|f| f.alt > start_alt + 50).collect::<Vec<Fix>>();
        let fixes = fixes.into_iter().map(|f| Rc::new(f)).collect::<Vec<Rc<Fix>>>();

        const DEGREE_BOUNDARY: f32 = 120.;  //turn this many degrees in
        const TIME_WINDOW: u32 = 15;        //this much time
        const CONNECT_TIME: u32 = 35;       //time one has to stop thermalling for it to be a glide
        const THERMAL_BACKSET: usize = 8;  //correcting factor for backwards looking thermal model should be roughly TIME_WINDOW / 2
        const TRY_TIME: u32 = 45;

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

        let mut buildup: Vec<Rc<Fix>> = vec![];
        let mut buildup_is_glide = true;
        let mut time_buildup = 0;
        let mut short_buildup: Vec<f32> = vec![];
        let mut prev_time = fixes.first().unwrap().timestamp;

        for (fix, change) in fixes.iter().zip(bearing_changes) {
            let delta_time = fix.timestamp - prev_time;
            prev_time = fix.timestamp;
            time_buildup += delta_time;
            short_buildup.push(change);
            buildup.push(Rc::clone(&fix));
            let total_degree_change = short_buildup.iter().sum::<f32>();
            if (total_degree_change / (time_buildup as f32)).abs() >= target {
                //We are turning!
                match buildup_is_glide {
                    true => { //We have just started turning!
                        buildup_is_glide = false;
                        let time_of_segment = buildup.last().unwrap().timestamp - buildup.first().unwrap().timestamp;
                        let segment = if time_of_segment <= CONNECT_TIME {
                            let mut prev_segment = match segments.pop() {
                                None => vec![],
                                Some(prev_segment) => prev_segment.inner().to_vec(),
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
                                    Segment::Thermal(_) => {
                                        if let Segment::Thermal(v) = segments.pop().unwrap() {
                                            v
                                        } else {
                                            panic!("unreachable")
                                        }
                                    },
                                    _ => vec![],
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

        segments.push(Segment::Glide(buildup));

        fn move_fixes_to_right_segments_by(segments: &mut Vec<Segment>, seconds: usize) {
            let mut segments = segments.into_iter();
            let mut curr_seg = segments.next().unwrap();
            for next_seg in segments {
                let first_time = curr_seg.mut_inner().first().unwrap().timestamp;
                let last_time = curr_seg.mut_inner().last().unwrap().timestamp;
                let time_delta = last_time - first_time;
                let log_time = time_delta as f32 / curr_seg.inner().len() as f32;

                let how_many_to_take = (seconds as f32 / log_time) as usize;
                let curr_size = curr_seg.inner().len();

                let mut fixes_to_move: Vec<Rc<Fix>> = if curr_seg.mut_inner().len() > how_many_to_take {
                    let last_index = curr_seg.mut_inner().len();
                    curr_seg.mut_inner().drain(last_index-how_many_to_take .. ).collect()
                } else {
                    curr_seg.mut_inner().drain(..).collect()
                };
                let mut debug = next_seg.inner().len();
                add_fixes_to_segment(next_seg, &mut fixes_to_move);
                debug = next_seg.inner().len();
                curr_seg = next_seg;
            }
        }

        fn add_fixes_to_segment(segment: &mut Segment, fixes: &mut Vec<Rc<Fix>>) {
            fixes.append(segment.mut_inner());
            segment.mut_inner().drain(..);
            segment.mut_inner().append(fixes)
        }

        fn replace_short_thermals_with_tries(segments: &mut Vec<Segment>, minimum_thermal_time: u32) {
            for i in 0..segments.len() {
                if segments[i].total_time() <= minimum_thermal_time {
                    let removed_inner = segments.remove(i).inner().clone();
                    segments.insert(i, Segment::Try(removed_inner))
                }
            }
        }

        move_fixes_to_right_segments_by(&mut segments, THERMAL_BACKSET);

        replace_short_thermals_with_tries(&mut segments, TRY_TIME);

        let segments = segments.into_iter().filter(|segment| !segment.inner().is_empty()).collect();


        let mut flight = Self {
            fixes,
            segments,
        };
        flight.combine_segments();

        flight
    }

    /// Combines subsequent relevant segments,
    /// after this the function will consist of Thermal, Glide, Thermal, ...
    /// with all thermals being below TRY_TIME
    fn combine_segments(&mut self) {
        let mut buildup = vec![];
        buildup.push(self.segments.remove(0));
        while !self.segments.is_empty() {
            let curr = self.segments.remove(0);
            match curr {
                Segment::Glide(mut curr_v) => {
                    match buildup.pop().unwrap() {
                        Segment::Glide(mut prev_v) => {
                            prev_v.append(&mut curr_v);
                            buildup.push(Segment::Glide(prev_v))
                        }
                        Segment::Thermal(v) => {
                            buildup.push(Segment::Thermal(v));
                            buildup.push(Segment::Glide(curr_v));
                        }
                        Segment::Try(mut prev_v) => {
                            prev_v.append(&mut curr_v);
                            buildup.push(Segment::Glide(prev_v))
                        }
                    }
                }
                Segment::Thermal(mut curr_v) => {
                    match buildup.pop().unwrap() {
                        Segment::Glide(prev_v) => {
                            buildup.push(Segment::Glide(prev_v));
                            buildup.push(Segment::Thermal(curr_v));
                        }
                        Segment::Thermal(mut prev_v) => {
                            prev_v.append(&mut curr_v);
                            buildup.push(Segment::Thermal(prev_v));
                        }
                        Segment::Try(prev_v) => {
                            buildup.push(Segment::Glide(prev_v));
                            buildup.push(Segment::Thermal(curr_v));
                        }
                    }
                }
                Segment::Try(mut curr_v) => {
                    match buildup.pop().unwrap() {
                        Segment::Glide(mut prev_v) => {
                            prev_v.append(&mut curr_v);
                            buildup.push(Segment::Glide(prev_v));
                        }
                        Segment::Thermal(prev_v) => {
                            buildup.push(Segment::Thermal(prev_v));
                            buildup.push(Segment::Glide(curr_v));
                        }
                        Segment::Try(mut prev_v) => {
                            prev_v.append(&mut curr_v);
                            buildup.push(Segment::Glide(prev_v));
                        }
                    }
                }
            }
        }
        self.segments = buildup
    }

    pub fn get_subflight(&self, from: u32, to: u32) -> Self {
        let fixes = self.fixes
            .iter()
            .filter(|f| (from..to).contains(&f.timestamp))
            .map(|f| Rc::clone(&f))
            .collect::<Vec<Rc<Fix>>>();
        let segments = self.segments.iter()
            .filter(|s| s.inner().last().unwrap().timestamp >= from && s.inner().first().unwrap().timestamp < to)
            .map(|mut s| {
                let inner = s.inner().clone().into_iter().filter(|fix| (from..to).contains(&fix.timestamp)).collect();
                match s {
                    Segment::Glide(_) => Segment::Glide(inner),
                    Segment::Thermal(_) => Segment::Thermal(inner),
                    Segment::Try(_) => Segment::Try(inner),
                }
            })
            .collect::<Vec<Segment>>();
        Self {
            fixes,
            segments,
        }
    }

    pub fn get_subflight_from_option(&self, from: Option<u32>, to: Option<u32>) -> Self {
        let from = match from {
            None => self.fixes.first().unwrap().timestamp,
            Some(from) => from,
        };
        let to = match to {
            None => self.fixes.last().unwrap().timestamp,
            Some(to) => to,
        };
        self.get_subflight(from, to)
    }

    pub fn thermal_percentage(&self) -> f32 {
        let thermal_length: f32 =
            self.segments.iter().map(
                |s| match s {
                    Segment::Thermal(v) => v.len() as f32,
                    _ => 0.
                }
            ).sum::<f32>();
        let total_length: f32 =
            self.segments.iter().map(
                |s| match s {
                    Segment::Glide(v) => v.len() as f32,
                    Segment::Thermal(v) => v.len() as f32,
                    Segment::Try(v) => v.len() as f32,
                }
            ).sum::<f32>();
        (thermal_length / total_length) * 100.
    }

    pub fn print_segments(&self, timezone: u8) {
        for segment in &self.segments {
            let inner = segment.inner();
            if inner.is_empty() {
                println!("\tEmpty")
            } else {

                let first = inner.first().unwrap();
                let last = inner.last().unwrap();
                let preffix_type = match segment {
                    Segment::Glide(_) => "Glide:",
                    Segment::Thermal(_) => "Thermal:",
                    Segment::Try(_) => "Try:",
                };
                let first_time = Time::from_hms(timezone+(first.timestamp / 3600) as u8, ((first.timestamp % 3600) / 60) as u8, (first.timestamp % 60) as u8);
                let last_time = Time::from_hms(timezone+(last.timestamp / 3600) as u8, ((last.timestamp % 3600) / 60) as u8, (last.timestamp % 60) as u8);


                println!("{}", preffix_type);
                println!("\t{}:{}:{} -> {}:{}:{}", first_time.hours, first_time.minutes, first_time.seconds, last_time.hours, last_time.minutes, last_time.seconds);
            }
        }
    }

    pub fn count_thermals(&self) -> usize {
        self.segments.iter().filter(|s| match s {
            Segment::Glide(_) => false,
            Segment::Thermal(_) => true,
            Segment::Try(_) => false,
        }).count()
    }

    pub(crate) fn total_time(&self) -> u32 {
        self.fixes.last().unwrap().timestamp - self.fixes.first().unwrap().timestamp
    }
}

pub enum Segment {
    Glide(Vec<Rc<Fix>>),
    Thermal(Vec<Rc<Fix>>),
    Try(Vec<Rc<Fix>>),
}

impl Segment {
    fn total_time(&self) -> u32 {
        let inner = match self {
            Segment::Glide(v) => v,
            Segment::Thermal(v) => v,
            Segment::Try(v) => v,
        };
        if inner.len() == 0 { return 0 }
        inner.last().unwrap().timestamp - inner.first().unwrap().timestamp
    }

    fn mut_inner(&mut self) -> &mut Vec<Rc<Fix>> {
        match self {
            Segment::Glide(v) => v,
            Segment::Thermal(v) => v,
            Segment::Try(v) => v,
        }
    }

    fn as_try(&self) -> Segment {
        Segment::Try(self.inner().clone())
    }

    pub fn inner(&self) -> &Vec<Rc<Fix>> {
        match self {
            Segment::Glide(v) => v,
            Segment::Thermal(v) => v,
            Segment::Try(v) => v,
        }
    }

    fn is_glide(&self) -> bool {
        match self {
            Segment::Thermal(_) => false,
            _ => true,
        }
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
        println!("Kawa 1\tSegments: {}", &flight.segments.len());
        println!("Kawa 1\tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    #[test]
    fn segmenting_2_sec() {
        let contents = parser::util::get_contents("examples/aat.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let n = 2;
        let every_nth = (n-1..fixes.len()).step_by(n).map(|i| fixes[i].clone()).collect::<Vec<Fix>>();
        let flight = Flight::make(every_nth);
        let target_percentage: f32 = 36.6;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("Kawa 2\tSegments: {}", &flight.segments.len());
        println!("Kawa 2\tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    #[test]
    fn segmenting_3_sec() {
        let contents = parser::util::get_contents("examples/aat.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let n = 3;
        let every_nth = (n-1..fixes.len()).step_by(n).map(|i| fixes[i].clone()).collect::<Vec<Fix>>();
        let flight = Flight::make(every_nth);
        let target_percentage: f32 = 36.6;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("Kawa 3\tSegments: {}", &flight.segments.len());
        println!("Kawa 3\tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    #[test]
    fn segmenting_4_sec() {
        let contents = parser::util::get_contents("examples/aat.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let n = 4;
        let every_nth = (n-1..fixes.len()).step_by(n).map(|i| fixes[i].clone()).collect::<Vec<Fix>>();
        let flight = Flight::make(every_nth);
        let target_percentage: f32 = 36.6;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("Kawa 4\tSegments: {}", &flight.segments.len());
        println!("Kawa 4\tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    #[test]
    fn segmenting_5_sec() {
        let contents = parser::util::get_contents("examples/aat.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let n = 5;
        let every_nth = (n-1..fixes.len()).step_by(n).map(|i| fixes[i].clone()).collect::<Vec<Fix>>();
        let flight = Flight::make(every_nth);
        let target_percentage: f32 = 36.6;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("Kawa 5\tSegments: {}", &flight.segments.len());
        println!("Kawa 5\tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    #[test]
    fn segmenting_1_sec_cx() {
        let contents = parser::util::get_contents("examples/CX.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let flight = Flight::make(fixes);
        let target_percentage: f32 = 34.8;
        let acceptance = 3.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("CX 1\tSegments: {}", &flight.segments.len());
        println!("CX 1\tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    #[test]
    fn segmenting_2_sec_cx() {
        let contents = parser::util::get_contents("examples/CX.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let n = 2;
        let every_nth = (n-1..fixes.len()).step_by(n).map(|i| fixes[i].clone()).collect::<Vec<Fix>>();
        let flight = Flight::make(every_nth);
        let target_percentage: f32 = 34.8;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("CX 2\tSegments: {}", &flight.segments.len());
        println!("CX 2\tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    #[test]
    fn segmenting_3_sec_cx() {
        let contents = parser::util::get_contents("examples/CX.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let n = 3;
        let every_nth = (n-1..fixes.len()).step_by(n).map(|i| fixes[i].clone()).collect::<Vec<Fix>>();
        let flight = Flight::make(every_nth);
        let target_percentage: f32 = 34.8;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("CX 3\tSegments: {}", &flight.segments.len());
        println!("CX 3\tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }

    #[test]
    fn segmenting_real_3_ke() {
        let contents = parser::util::get_contents("examples/ast.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let flight = Flight::make(fixes);
        let target_percentage: f32 = 45.1;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        println!("KE \tSegments: {}", &flight.segments.len());
        println!("KE \tPercentage: {}", &flight.thermal_percentage());
        assert!(range.contains(&flight.thermal_percentage()));
    }
}

