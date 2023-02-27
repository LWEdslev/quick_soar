use std::rc::Rc;
use igc::util::Time;
use crate::analysis::segmenting::Flight;
use crate::parser::pilot_info::PilotInfo;
use crate::parser::task::{Task, TaskComponent, TaskType};
use crate::parser::util::Fix;

type FloatMeters = f32;
type Meters = i16;
type Seconds = u32;
type Kph = f32;
type Mps = f32;
type Percentage = f32;

pub struct CalculatedFix {
    distance: FloatMeters,
    alt_gain: Meters,
    time_delta: Seconds,
    timestamp: Seconds,

}

impl CalculatedFix {
    fn new(from: &Fix, to: &Fix) -> Self {
        let distance = from.distance_to(to);
        let alt_gain = from.alt - to.alt;
        let time_delta = to.timestamp - from.timestamp;
        Self {
            distance,
            alt_gain,
            time_delta,
            timestamp: from.timestamp,
        }
    }
}

pub struct Calculation {
    pub legs: Vec<Option<Flight>>,
    pub total_flight: Flight,
    pub task: Task,
    pub calculated_fixes: Vec<Rc<CalculatedFix>>, //fixes for entire flight
    pub calculated_legs: Vec<Option<Vec<Rc<CalculatedFix>>>>, //fixes of legs for entire
    pub pilot_info: PilotInfo,
}

impl Calculation {
    pub fn new(task: Task, flight: Flight, pilot_info: PilotInfo, start_time: u32) -> Calculation {
        let fixes = flight.fixes.iter().map(|f| Rc::clone(&f)).collect::<Vec<Rc<Fix>>>();

        let calculated_fixes = Calculation::calculate_fixes(&fixes);

        let legs = Calculation::make_legs(&fixes, &task, start_time, &flight);

        let calculated_legs = legs.iter().map(|opt| match opt {
            None => None,
            Some(inner) => Some(Calculation::calculate_fixes(&inner.fixes)),
        }).collect();

        Self {
            legs,
            total_flight: flight,
            task,
            calculated_fixes,
            calculated_legs,
            pilot_info,
        }
    }

    pub fn speed(&self, task_piece: TaskPiece) -> Kph { todo!() }

    pub fn glide_ratio(&self, task_piece: TaskPiece) -> Kph { todo!() }

    pub fn excess_distance(&self, task_piece: TaskPiece) -> Percentage { todo!() }

    pub fn climb_rate(&self, task_piece: TaskPiece) -> Mps { todo!() }

    pub fn start_time(&self, task_piece: TaskPiece) -> Time { todo!() }

    pub fn finish_time(&self, task_piece: TaskPiece) -> Time { todo!() }

    pub fn start_alt(&self, task_piece: TaskPiece) -> Time { todo!() }

    pub fn climb_ground_speed(&self, task_piece: TaskPiece) -> Kph { todo!() }

    pub fn glide_speed(&self, task_piece: TaskPiece) -> Kph { todo!() }

    pub fn climb_percentage(&self, task_piece: TaskPiece) -> Percentage { todo!() }

    fn calculate_fixes(fixes: &Vec<Rc<Fix>>) -> Vec<Rc<CalculatedFix>> {
        if fixes.is_empty() {return vec![]};
        let mut fixes = fixes.iter();
        let mut prev_fix = fixes.next().unwrap();
        fixes.map(|curr_fix| {
            let calc_fix = CalculatedFix::new(prev_fix, curr_fix);
            prev_fix = curr_fix;
            Rc::new(calc_fix)
        }).collect::<Vec<Rc<CalculatedFix>>>()
    }

    fn make_legs(fixes: &Vec<Rc<Fix>>, task: &Task, start_time: u32, flight: &Flight) -> Vec<Option<Flight>> {
        match task.task_type {
            TaskType::AST => {
                let mut turnpoints = task.points.iter();
                let start_point = turnpoints.next().unwrap();
                let mut fixes = fixes.iter().filter(|fix| fix.timestamp >= start_time); //get fixes after start
                let start_fix = fixes.next();
                let mut inside_turnpoints = turnpoints.map(|turnpoint| match turnpoint {
                    TaskComponent::Start(_) => {panic!("unexpected start token")}
                    _ => {
                        fixes.clone().filter(|fix| turnpoint.inner().is_inside(fix))
                            .map(|f| Rc::clone(f))
                            .collect::<Vec<Rc<Fix>>>()
                    }
                }).collect::<Vec<Vec<Rc<Fix>>>>();
                inside_turnpoints.insert(0, vec![Rc::clone(start_fix.unwrap())]); //add start as the first turnpoint

                let mut curr_time = Some(inside_turnpoints.first().unwrap().first().unwrap().timestamp);
                let start_time = inside_turnpoints.remove(0).first().unwrap().timestamp;
                let mut leg_times = inside_turnpoints.iter().map(|in_tp| {
                    if curr_time.is_none() { return None } //landout previously
                    let after_prev = in_tp.iter().filter(|fix| fix.timestamp >= curr_time.unwrap()).collect::<Vec<&Rc<Fix>>>();
                    if after_prev.is_empty() { //landout
                        None
                    } else {
                        let found = Some(after_prev.first().unwrap().timestamp);
                        curr_time = found;
                        found
                    }
                }).collect::<Vec<Option<u32>>>();
                leg_times.insert(0, Some(start_time));

                let legs = leg_times.windows(2).map(|window| {
                    match (window[0], window[1]) {
                        (Some(start), Some(end)) => Some(flight.get_subflight(start, end)),
                        _ => None,
                    }
                }).collect::<Vec<Option<Flight>>>();

                legs

            }
            TaskType::AAT(_) => { vec![] } //TODO: Add AAT support
        }
    }
}

pub enum TaskPiece {
    EntireTask,
    Leg(usize),
}