use std::rc::Rc;
use igc::util::Time;
use crate::analysis::segmenting::{Flight, Segment};
use crate::parser::pilot_info::PilotInfo;
use crate::parser::task::{Task, TaskComponent, TaskType};
use crate::parser::util::Fix;

type FloatMeters = f32;
type Meters = i16;
type Seconds = u32;
type Kph = f32;
type Mps = f32;
type Percentage = f32;


pub struct Calculation {
    pub legs: Vec<Option<Flight>>,
    pub total_flight: Flight,
    pub task: Task,
    pub pilot_info: PilotInfo,
    speed: Option<Kph>,
    distance: Option<FloatMeters>,
    qfe_alt: i16,
}

impl Calculation {
    pub fn new(
        task: Task,
        flight: Flight,
        pilot_info: PilotInfo,
        start_time: Option<Seconds>,
        speed: Option<Kph>,
        distance: Option<FloatMeters>,
    ) -> Calculation {
        let fixes = flight.fixes.iter().map(Rc::clone).collect::<Vec<Rc<Fix>>>();

        let qfe_alt = fixes[0].alt_igc;

        let legs = Calculation::make_legs(&fixes, &task, start_time, &flight);

        let last_time = if legs.last().is_some() && legs.last().unwrap().is_some() && legs.last().as_ref().unwrap().as_ref().unwrap().fixes.last().is_some() {
            legs.last().as_ref().unwrap().as_ref().unwrap().fixes.last().unwrap().timestamp
        } else {
            flight.fixes.last().as_ref().unwrap().timestamp
        };

        let flight = flight.get_subflight_from_option(start_time, Some(last_time));

        Self {
            legs,
            total_flight: flight,
            task,
            pilot_info,
            speed,
            distance,
            qfe_alt
        }
    }

    pub fn speed(&self, task_piece: TaskPiece) -> Option<Kph> {
        match task_piece {
            TaskPiece::EntireTask => {
                self.speed
            }
            TaskPiece::Leg(leg_number) => {
                if leg_number >= self.legs.len() {return None}
                let leg = self.legs[leg_number].as_ref();
                leg?;
                let points = &self.task.points;
                let time = leg.unwrap().total_time();
                match self.task.task_type {
                    TaskType::AAT(_) => {

                        let distance = leg.unwrap();
                        let distance = distance.fixes.first().unwrap().distance_to(distance.fixes.last().unwrap());
                        Some(3.6 * distance / (time as f32))
                    }
                    TaskType::AST => {
                        let distance = points[leg_number].inner().distance_to(points[leg_number + 1].inner());
                        Some(3.6 * distance / (time as f32))
                    }
                }

            }
        }
    }

    pub fn glide_ratio(&self, task_piece: TaskPiece) -> Option<Kph> {
        let segments = match task_piece {
            TaskPiece::EntireTask => {
                Some(&self.total_flight.segments)
            }
            TaskPiece::Leg(leg_number) => {
                if self.legs.len() <= leg_number || self.legs[leg_number].is_none() {return None};
                Some(&self.legs[leg_number].as_ref().unwrap().segments)
            }
        };
        let segments = segments?;
        let glides = segments.iter().filter(|s| match s {
            Segment::Thermal(_) => false,
            Segment::Glide(_) => true,
            Segment::Try(_) => true,
        });
        let (dist, alt_loss) = glides.into_iter().map(|s| {
            let inner = s.inner();
            let dist = inner.windows(2).map(|w| {
                let first = &w[0];
                let second = &w[1];
                first.distance_to(second)
            }).sum::<f32>();
            let alt = inner.windows(2).map(|w| {
                let first = &w[0];
                let second = &w[1];
                first.alt - second.alt
            }).sum::<i16>();
            (dist, alt)
        })
            .fold((0f32,0i16), |acc, (dist, alt)| (acc.0 + dist, acc.1 + alt));
        Some(dist / (alt_loss as f32))
    }

    pub fn distance(&self, task_piece: TaskPiece) -> Option<FloatMeters> {
        match task_piece {
            TaskPiece::EntireTask => {
                self.distance
            }
            TaskPiece::Leg(leg_number) => {
                let leg = self.legs.get(leg_number)?;
                let leg = leg.as_ref()?;
                let first = leg.fixes.first().unwrap();
                let last = leg.fixes.last().unwrap();
                Some(first.distance_to(last))
            }
        }
    }

    pub fn excess_distance(&self, task_piece: TaskPiece) -> Option<Percentage> {
        let (flight_part, task_dist) = match task_piece {
            TaskPiece::EntireTask => {
                let legs = &self.legs;
                let task_dist = legs.iter().map(|leg| {
                    let leg = leg.as_ref();
                    if leg.is_none() { 0. } else {
                        let leg = leg.unwrap();
                        let first = leg.fixes.first().unwrap();
                        let last = leg.fixes.last().unwrap();
                        first.distance_to(last)
                    }
                }).sum::<FloatMeters>();
                (&self.total_flight, task_dist)
            }
            TaskPiece::Leg(leg_number) => {
                let leg = self.legs.get(leg_number)?.as_ref()?;
                let task_dist = {
                    let first = leg.fixes.first()?;
                    let last = leg.fixes.last()?;
                    first.distance_to(last)
                };
                (leg, task_dist)
            }
        };

        let glides = flight_part.segments.iter().filter(|seg| match seg {
            Segment::Glide(_) => true,
            _ => false,
        });

        let total_glide_distance = glides.map(|glide| {
            let glide = glide.inner();
            glide.windows(2).map(|w| {
                let curr = &w[0];
                let next = &w[1];
                curr.distance_to(next)
            }).sum::<FloatMeters>()
        }).sum::<FloatMeters>();

        let thermals = flight_part.segments.iter().filter(|seg| match seg {
            Segment::Thermal(_) => true,
            _ => false,
        });

        let total_thermal_distance = thermals.map(|thermal| {
            let thermal = thermal.inner();
            let first = thermal.first().unwrap();
            let last = thermal.last().unwrap();
            first.distance_to(last)
        }).sum::<FloatMeters>();

        Some((100. * (total_glide_distance + total_thermal_distance) / task_dist) - 100.)
    }

    pub fn climb_rate(&self, task_piece: TaskPiece) -> Option<Mps> {
        let flight_part = match task_piece {
            TaskPiece::EntireTask => {
                &self.total_flight
            }
            TaskPiece::Leg(leg_number) => {
                
                self.legs.get(leg_number)?.as_ref()?
            }
        };

        let climbs = flight_part.segments.iter().filter(|seg| match seg {
            Segment::Thermal(_) => true,
            _ => false,
        });

        let (total_alt_gain, total_climb_time) = climbs.map(|climb| {
            let climb = climb.inner();
            let first = climb.first().unwrap();
            let last = climb.last().unwrap();
            let delta_time = last.timestamp - first.timestamp;
            let alt_gain = last.alt - first.alt;
            (alt_gain, delta_time)
        }).fold((0, 0), |(alt_acc, time_acc), (alt, time)| (alt_acc + alt, time_acc + time));
        if total_climb_time == 0 {return None};
        Some((total_alt_gain as f32) / (total_climb_time as f32))
    }

    pub fn start_time(&self, task_piece: TaskPiece) -> Option<Time> {
        match task_piece {
            TaskPiece::EntireTask => {
                if self.total_flight.fixes.is_empty() { return None };
                let time_in_seconds = self.total_flight.fixes[0].timestamp;
                Some(Time::from_hms((time_in_seconds / 3600) as u8, ((time_in_seconds % 3600) / 60) as u8, (time_in_seconds % 60) as u8))
            }
            TaskPiece::Leg(leg_number) => {
                if self.legs.get(leg_number).is_none() || self.legs.get(leg_number).unwrap().is_none() { return None };
                let time_in_seconds = self.legs[leg_number].as_ref().unwrap().fixes.first().unwrap().timestamp;
                Some(Time::from_hms((time_in_seconds / 3600) as u8, ((time_in_seconds % 3600) / 60) as u8, (time_in_seconds % 60) as u8))
            }
        }
    }

    pub fn finish_time(&self, task_piece: TaskPiece) -> Option<Time> {
        let flight = match task_piece {
            TaskPiece::EntireTask => Some(&self.total_flight),
            TaskPiece::Leg(leg_number) => self.legs.get(leg_number)?.as_ref(),
        };

        let time_in_seconds = flight?.fixes.last()?.timestamp;
        Some(Time::from_hms((time_in_seconds / 3600) as u8, ((time_in_seconds % 3600) / 60) as u8, (time_in_seconds % 60) as u8))
    }

    pub fn start_alt(&self, task_piece: TaskPiece) -> Option<Meters> {
        match task_piece {
            TaskPiece::EntireTask => {
                let fix = self.total_flight.fixes.first()?;
                Some(fix.alt_igc)
            }
            TaskPiece::Leg(leg_number) => {
                let leg = (self.legs.get(leg_number))?.as_ref()?;
                Some(leg.fixes.first()?.alt_igc)
            }
        }
    }

    pub fn climb_ground_speed(&self, task_piece: TaskPiece) -> Option<Kph> {
        self.get_avg_speed_of_segment(task_piece, false)
    }

    pub fn glide_speed(&self, task_piece: TaskPiece) -> Option<Kph> {
        self.get_avg_speed_of_segment(task_piece, true)
    }

    pub fn climb_percentage(&self, task_piece: TaskPiece) -> Option<Percentage> {
        match task_piece {
            TaskPiece::EntireTask => {
                Some(self.total_flight.thermal_percentage())
            }
            TaskPiece::Leg(leg_number) => {
                let leg = self.legs.get(leg_number)?;
                let leg = leg.as_ref()?;
                Some(leg.thermal_percentage())
            }
        }
    }

    pub fn glide_distance(&self, task_piece: TaskPiece) -> Option<FloatMeters> {
        let flight = match task_piece {
            TaskPiece::EntireTask => Some(&self.total_flight),
            TaskPiece::Leg(leg_number) => self.legs.get(leg_number)?.as_ref(),
        };

        let each_glide_distance = flight?.segments.iter().filter(|seg| match seg {
            Segment::Thermal(_) => false,
            _ => true,
        }).map(|glide| {
            let inner = glide.inner();
            if inner.is_empty() { return 0. }
            let first = inner.first().unwrap();
            let last = inner.last().unwrap();
            first.distance_to(last)
        }).collect::<Vec<FloatMeters>>();

        Some(each_glide_distance.iter().sum::<FloatMeters>() / each_glide_distance.len() as f32)
    }

    pub fn thermal_height_loss(&self, task_piece: TaskPiece) -> Option<Percentage> {
        let flight = match task_piece {
            TaskPiece::EntireTask => Some(&self.total_flight),
            TaskPiece::Leg(leg_number) => self.legs.get(leg_number)?.as_ref(),
        };

        let alt_gains_and_loss = &flight?.segments.iter().filter(|seg| match seg {
                Segment::Thermal(_) => true,
                _ => false,
            }
        ).map(|thermal| {
            let inner = thermal.inner();
            let _first = inner.first().unwrap();
            let _last = inner.last().unwrap();
            let alt_gain = inner.windows(2).map(|w| {
                let curr = &w[0];
                let next = &w[1];
                (next.alt - curr.alt).max(0)
            }).sum::<Meters>();
            let alt_loss = inner.windows(2).map(|w| {
                let curr = &w[0];
                let next = &w[1];
                (curr.alt - next.alt).max(0)
            }).sum::<Meters>();

            (alt_gain, alt_loss)
        }).collect::<Vec<(Meters, Meters)>>();

        let total_alt_gain = alt_gains_and_loss.iter().map(|s| s.0).sum::<Meters>() as FloatMeters;
        let total_alt_loss = alt_gains_and_loss.iter().map(|s| s.1).sum::<Meters>() as FloatMeters;
        if total_alt_gain.is_nan() {return None};
        Some(100. * total_alt_loss / total_alt_gain)
    }

    // pub fn circling_radius(&self, task_piece: TaskPiece) -> Option<FloatMeters> { }

    pub fn wind_thermal_gain(&self, task_piece: TaskPiece) -> Option<Percentage> {
        fn find_thermal_gain_over_leg(leg: &Flight) -> Option<Percentage> {
            let last_fix = leg.fixes.last()?.as_ref();
            let thermals = leg.segments.iter().filter(|seg| match seg {
                Segment::Thermal(_) => true,
                _ => false,
            }).collect::<Vec<&Segment>>();
            let thermal_gain = thermals.iter().map(move |thermal| {
                let first_thermal_fix = thermal.inner().first().unwrap();
                let last_thermal_fix = thermal.inner().last().unwrap();
                let first_dist = first_thermal_fix.distance_to(last_fix);
                let last_dist = last_thermal_fix.distance_to(last_fix);
                first_dist - last_dist
            }).sum::<FloatMeters>();
            Some(thermal_gain)
        }

        fn leg_dist(leg: &Flight) -> Option<FloatMeters> {
            let first_fix = leg.fixes.first()?.as_ref();
            let last_fix = leg.fixes.last()?.as_ref();
            Some(first_fix.distance_to(last_fix))
        }

        match task_piece {
            TaskPiece::EntireTask => {
                let total_thermal_gain = self.legs.iter().filter_map(|leg| match leg {
                    None => None,
                    Some(leg) => find_thermal_gain_over_leg(leg),
                }).sum::<FloatMeters>();

                let total_dist = self.legs.iter().filter_map(|leg| match leg {
                    None => None,
                    Some(leg) => leg_dist(leg),
                }).sum::<FloatMeters>();

                Some((total_thermal_gain * 100.) / total_dist)
            }
            TaskPiece::Leg(leg_num) => {
                let leg = self.legs.get(leg_num)?.as_ref()?;
                let thermal_gain = find_thermal_gain_over_leg(leg)?;
                let dist = leg_dist(leg)?;
                Some((100. * thermal_gain)/dist)
            }
        }
    }

    pub fn time_below_500m_qfe(&self, task_piece: TaskPiece) -> Option<Percentage> {
        impl Flight {
            fn time_below_500(&self, qfe_alt: Meters) -> Option<Percentage> {
                if self.fixes.is_empty() { return None };
                let all_fixes = self.fixes.len() as f32;
                let low_fixes = self.fixes.iter().filter(|fix| fix.alt_igc <= qfe_alt + 500).count() as f32;
                Some((low_fixes * 100.) / all_fixes)
            }
        }
        match task_piece {
            TaskPiece::EntireTask => {
                self.total_flight.time_below_500(self.qfe_alt)
            }
            TaskPiece::Leg(leg_num) => {
                self.legs.get(leg_num)?.as_ref()?.time_below_500(self.qfe_alt)
            }
        }
    }

    pub fn get_pilot_info(&self) -> &PilotInfo {
        &self.pilot_info
    }

    pub fn get_task(&self) -> &Task {
        &self.task
    }

    fn get_avg_speed_of_segment(&self, task_piece: TaskPiece, is_glide: bool) -> Option<Kph> {
        let flight = match task_piece {
            TaskPiece::EntireTask => {
                Some(&self.total_flight)
            }
            TaskPiece::Leg(leg_number) => {
                let leg = &self.legs.get(leg_number)?;
                leg.as_ref()
            }
        };

        let flight = flight?;
        let climbs = flight.segments.iter().filter(|seg| match seg {
            Segment::Thermal(_) => !is_glide,
            _ => is_glide,
        });
        let (total_climb_dist, total_climb_time) = climbs.map(|seg| {
            let dist = seg.inner().windows(2).map(|w| {
                let curr = &w[0];
                let next = &w[1];
                curr.distance_to(next)
            }).sum::<FloatMeters>();
            let first = seg.inner().first().unwrap();
            let last = seg.inner().last().unwrap();
            let time = last.timestamp - first.timestamp;
            (dist, time)
        }).fold((0.,0), |(acc_dist, acc_time), (dist, time)| (acc_dist + dist, acc_time + time));
        if total_climb_time == 0 { return None };
        Some(3.6 * (total_climb_dist / (total_climb_time as f32)))
    }

    fn make_legs(fixes: &Vec<Rc<Fix>>, task: &Task, start_time: Option<Seconds>, flight: &Flight) -> Vec<Option<Flight>> {
        let mut turnpoints = task.points.iter();
        let _start_point = turnpoints.next().unwrap();
        if start_time.is_none() { return turnpoints.map(|_| None).collect::<Vec<Option<Flight>>>()}; //No start should give no legs
        let start_time = start_time.unwrap();
        let mut fixes_iter = fixes.iter().filter(|fix| fix.timestamp >= start_time); //get fixes after start
        let start_fix = fixes_iter.next();
        let mut inside_turnpoints = turnpoints.map(|turnpoint| match turnpoint {
            TaskComponent::Start(_) => {panic!("unexpected start token")}
            _ => {
                fixes_iter.clone().filter(|fix| turnpoint.inner().is_inside(fix))
                    .map(Rc::clone)
                    .collect::<Vec<Rc<Fix>>>()
            }
        }).collect::<Vec<Vec<Rc<Fix>>>>();
        inside_turnpoints.insert(0, vec![Rc::clone(start_fix.unwrap())]); //add start as the first turnpoint

        let mut curr_time = Some(inside_turnpoints.first().unwrap().first().unwrap().timestamp);
        let start_time = inside_turnpoints.remove(0).first().unwrap().timestamp;
        let mut leg_times = inside_turnpoints.iter().map(|in_tp| {
            curr_time?; //landout previously
            let after_prev = in_tp.iter().filter(|fix| fix.timestamp >= curr_time.unwrap()).collect::<Vec<&Rc<Fix>>>();
            if after_prev.is_empty() { //landout
                None
            } else {
                let found = Some(after_prev.first().unwrap().timestamp);
                curr_time = found;
                found
            }
        }).collect::<Vec<Option<Seconds>>>();
        leg_times.insert(0, Some(start_time));
        inside_turnpoints.insert(0, vec![Rc::clone(start_fix.unwrap())]); //add start as the first turnpoint

        match task.task_type {
            TaskType::AST => {
                let legs = leg_times.windows(2).enumerate().map(|(i, window)| {
                    match (window[0], window[1]) {
                        (Some(start), Some(end)) => Some(flight.get_subflight(start, end)),
                        (Some(start), None) => {
                            let best_fix = fixes.iter()
                                .filter(|fix| fix.timestamp >= start)
                                .map(|fix| (fix, fix.distance_to_tp(task.points[i].inner())))
                                .max_by(|(_x_fix, x_dist),(_y_fix, y_dist)| x_dist.total_cmp(y_dist)).unwrap().0;
                            Some(flight.get_subflight(start, best_fix.timestamp))
                        },
                        _ => None,
                    }
                }).collect::<Vec<Option<Flight>>>();

                legs

            }
            TaskType::AAT(_) => {
                //Getting ordered non-overlapping of consecutive sectors inside turnpoints
                let mut inside_turnpoints = inside_turnpoints.iter().zip(leg_times.windows(2)).map(|(v, leg_time)| {
                    let start_leg = leg_time[0];
                    let end_leg = leg_time[1];
                    v.iter().filter(move |fix|
                        start_leg.is_some() && start_leg.unwrap() <= fix.timestamp
                    &&  end_leg.is_some() && end_leg.unwrap() > fix.timestamp
                    ).map(Rc::clone).collect::<Vec<Rc<Fix>>>()
                }).collect::<Vec<Vec<Rc<Fix>>>>();
                let finish_fix = fixes.iter().filter(|fix| match leg_times.last() {
                    Some(Some(time)) => time == &fix.timestamp,
                    _ => false,
                }).next();
                inside_turnpoints.push(match finish_fix {
                    None => vec![],
                    Some(fix) => vec![Rc::clone(fix)],
                });
                //at this point |inside_turnpoints| == |task.points|
                let start_fixes = inside_turnpoints.remove(0);
                if start_fixes.len() == 0 { return inside_turnpoints.iter().map(|i| None).collect::<Vec<Option<Flight>>>() }
                inside_turnpoints.pop();
                let mut prev_optimal = Some(Rc::clone(start_fixes.first().unwrap()));
                assert_eq!(inside_turnpoints.len(), task.points.windows(3).count());
                let mut leg_times = task.points.windows(3).zip(inside_turnpoints.iter()).map(|(window, fixes)| {
                    match &prev_optimal {
                        None => None,
                        Some(prev_optimal_inner) => {
                            let (_, _, next) = (&window[0], &window[1], &window[2]);
                            let best_fix = fixes.iter()
                                .max_by(|x, y| {
                                (x.distance_to_tp(next.inner()) + x.distance_to(prev_optimal_inner))
                                    .total_cmp(&(y.distance_to_tp(next.inner()) + y.distance_to(prev_optimal_inner)))
                            });

                            match best_fix {
                                None => {
                                    prev_optimal = None;
                                    None
                                },
                                Some(best_fix) => {
                                    prev_optimal = Some(Rc::clone(best_fix));
                                    Some(best_fix.timestamp)
                                }
                            }
                        }
                    }


                }).collect::<Vec<Option<Seconds>>>();
                leg_times.insert(0, Some(start_time)); //add start
                leg_times.push(finish_fix.map(|fix| fix.timestamp));
                let legs = leg_times.windows(2).map(|window| {
                    match (window[0], window[1]) {
                        (Some(start), Some(end)) => Some(flight.get_subflight(start, end)),
                        _ => None,
                    }
                }).collect::<Vec<Option<Flight>>>();

                

                legs.into_iter().map(|leg| match leg {
                    None => None,
                    Some(leg) => {
                        match !leg.fixes.is_empty() {
                            true => Some(leg),
                            false => None,
                        }
                    }
                }).collect::<Vec<Option<Flight>>>()
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum TaskPiece {
    EntireTask,
    Leg(usize),
}