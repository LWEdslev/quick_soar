use std::rc::Rc;
use crate::parser::util::Fix;
use crate::analysis;

pub struct Flight {
    pub fixes: Vec<Rc<Fix>>,
    pub segments: Vec<Segment>
}

impl Flight {
    pub fn make(mut fixes: Vec<Fix>) -> Option<Self> {
        let mut segments: Vec<Segment> = vec![];
        fixes.retain(|f| f.is_valid());
        let mut prev_sound_fix = fixes.get(0)?.clone();
        fixes.retain(|f| {
            if f.timestamp < prev_sound_fix.timestamp || f.speed_to(&prev_sound_fix).abs() > 200. {
                false
            } else {
                prev_sound_fix = f.clone();
                true
            }});
        let fixes = fixes.into_iter().map(Rc::new).collect::<Vec<Rc<Fix>>>();

        const DEGREE_BOUNDARY: f32 = 150.;  //turn this many degrees in
        const TIME_WINDOW: u32 = 15;        //this much time
        const CONNECT_TIME: u32 = 35;       //time one has to stop thermalling for it to be a glide
        const THERMAL_BACKSET: usize = 8;  //correcting factor for backwards looking thermal model should be roughly TIME_WINDOW / 2
        const TRY_TIME: u32 = 45;

        let target = DEGREE_BOUNDARY / TIME_WINDOW as f32;
        let mut prev_fix = fixes.get(0)?;
        let mut curr_fix = fixes.get(0)?;
        let mut next_fix = fixes.get(0)?;
        let bearing_changes = fixes.iter().map(|f| {
            prev_fix = curr_fix;
            curr_fix = next_fix;
            next_fix = f;
            analysis::util::bearing_change(prev_fix, curr_fix, next_fix)
        }).collect::<Vec<f32>>();

        let mut buildup: Vec<Rc<Fix>> = vec![];
        let mut buildup_is_glide = true;
        let mut time_buildup = 0;
        let mut short_buildup: Vec<(u32, f32)> = vec![];
        let mut prev_time = fixes.first()?.timestamp;

        for (fix, change) in fixes.iter().zip(bearing_changes) {
            let delta_time = fix.timestamp.checked_sub(prev_time).unwrap_or(1);
            prev_time = fix.timestamp;
            time_buildup += delta_time;
            short_buildup.push((fix.timestamp, change));
            buildup.push(Rc::clone(fix));
            let total_degree_change = short_buildup.iter().map(|b|b.1).sum::<f32>();
            if (total_degree_change / (time_buildup as f32)).abs() >= target { //We are turning!
                if buildup_is_glide { //We have just started turning!
                    buildup_is_glide = false;
                    let time_of_segment = buildup.last()?.timestamp - buildup.first()?.timestamp;
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
                }
            } else {
                //We are going straight!
                if !buildup_is_glide { //We just stopped turning!
                    buildup_is_glide = true;
                    let mut prev_segment = match segments.last() {
                        None => vec![],
                        Some(Segment::Thermal(_)) => {
                            if let Segment::Thermal(v) = segments.pop()? {
                                v
                            } else {
                                panic!("unreachable")
                            }
                        },
                        Some(_) => vec![],
                    };
                    prev_segment.append(&mut buildup.clone());
                    let segment = Segment::Thermal(prev_segment);
                    segments.push(segment);
                    buildup.clear();
                }
            }

            time_buildup = short_buildup.last()?.0 - short_buildup.first()?.0;
            while time_buildup >= TIME_WINDOW {
                short_buildup.remove(0);
                time_buildup = short_buildup.last()?.0 - short_buildup.first()?.0;
            }
        }

        segments.push(Segment::Glide(buildup));

        fn move_fixes_to_right_segments_by(segments: &mut Vec<Segment>, seconds: usize) {
            if segments.is_empty() { return };
            let mut segments = segments.iter_mut();
            let mut curr_seg = segments.next().expect("unreachable");
            for next_seg in segments {
                if curr_seg.inner().is_empty() { curr_seg = next_seg; continue; }
                let first_time = curr_seg.mut_inner().first().expect("unreachable").timestamp;
                let last_time = curr_seg.mut_inner().last().expect("unreachable").timestamp;
                let time_delta = last_time.checked_sub(first_time);
                if time_delta.is_none() { return };
                let time_delta = time_delta.expect("unreachable");
                let log_time = time_delta as f32 / curr_seg.inner().len() as f32;

                let how_many_to_take = (seconds as f32 / log_time) as usize;

                let mut fixes_to_move: Vec<Rc<Fix>> = if curr_seg.mut_inner().len() > how_many_to_take {
                    let last_index = curr_seg.mut_inner().len();
                    curr_seg.mut_inner().drain(last_index-how_many_to_take .. ).collect()
                } else {
                    curr_seg.mut_inner().drain(..).collect()
                };
                add_fixes_to_segment(next_seg, &mut fixes_to_move);
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
        
        Some(flight)
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
                    match buildup.pop().expect("unreachable") {
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
                    match buildup.pop().expect("unreachable") {
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
                    match buildup.pop().expect("unreachable") {
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

    pub fn get_subflight(&self, from: u32, to: u32) -> Option<Self> {
        let fixes = self.fixes
            .iter()
            .filter(|f| (from..to).contains(&f.timestamp))
            .map(Rc::clone)
            .collect::<Vec<Rc<Fix>>>();
        let segments = self.segments.iter()
            .filter(|s| !s.inner().is_empty()
                && s.inner().last().expect("unreachable").timestamp >= from
                && s.inner().first().expect("unreachable").timestamp < to)
            .map(|s| {
                let inner = s.inner().clone().into_iter().filter(|fix| (from..to).contains(&fix.timestamp)).collect();
                match s {
                    Segment::Glide(_) => Segment::Glide(inner),
                    Segment::Thermal(_) => Segment::Thermal(inner),
                    Segment::Try(_) => Segment::Try(inner),
                }
            })
            .collect::<Vec<Segment>>();
        Some(Self {
            fixes,
            segments,
        })
    }

    pub fn get_subflight_from_option(&self, from: Option<u32>, to: Option<u32>) -> Option<Self> {
        let from = match from {
            None => self.fixes.first()?.timestamp,
            Some(from) => from,
        };
        let to = match to {
            None => self.fixes.last()?.timestamp,
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

    pub fn count_thermals(&self) -> usize {
        self.segments.iter().filter(|s| match s {
            Segment::Glide(_) => false,
            Segment::Thermal(_) => true,
            Segment::Try(_) => false,
        }).count()
    }

    pub(crate) fn total_time(&self) -> u32 {
        if self.fixes.is_empty() { return 0 };
        self.fixes.last().expect("unreachable").timestamp - self.fixes.first().expect("unreachable").timestamp
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
        if inner.is_empty() { return 0 }
        inner.last().expect("unreachable").timestamp - inner.first().expect("unreachable").timestamp
    }

    fn mut_inner(&mut self) -> &mut Vec<Rc<Fix>> {
        match self {
            Segment::Glide(v) => v,
            Segment::Thermal(v) => v,
            Segment::Try(v) => v,
        }
    }

    pub fn inner(&self) -> &Vec<Rc<Fix>> {
        match self {
            Segment::Glide(v) => v,
            Segment::Thermal(v) => v,
            Segment::Try(v) => v,
        }
    }
}
