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

struct CalculatedFix {
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

struct Calculation {
    legs: Vec<Flight>,
    total_flight: Flight,
    task: Task,
    calculated_fixes: Vec<Rc<CalculatedFix>>, //fixes for entire flight
    calculated_legs: Vec<Vec<Rc<CalculatedFix>>>, //fixes of legs for entire
    pilot_info: PilotInfo,
}

impl Calculation {
    fn new(task: Task, flight: Flight, pilot_info: PilotInfo) -> Calculation {
        let fixes = flight.fixes.iter().map(|f| Rc::clone(&f)).collect::<Vec<Rc<Fix>>>();

        fn calculate_fixes(fixes: &Vec<Rc<Fix>>) -> Vec<Rc<CalculatedFix>> {
            let mut fixes = fixes.iter();
            let mut prev_fix = fixes.next().unwrap();
            fixes.map(|curr_fix| {
                let calc_fix = CalculatedFix::new(prev_fix, curr_fix);
                prev_fix = curr_fix;
                Rc::new(calc_fix)
            }).collect::<Vec<Rc<CalculatedFix>>>()
        }

        let calculated_fixes = calculate_fixes(&fixes);

        fn make_legs(fixes: Vec<Rc<Fix>>, task: Task, start_time: u32) -> Vec<Flight> {
            match task.task_type {
                TaskType::AST => {
                    let mut turnpoints = task.points.iter();
                    let start_point = turnpoints.next().unwrap();
                    let mut fixes = fixes.iter().filter(|fix| fix.timestamp >= start_time); //get fixes after start
                    let inside_turnpoints = turnpoints.map(|turnpoint| match turnpoint {
                        TaskComponent::Start(_) => {panic!("unexpected start token")}
                        _ => {
                            fixes.clone().filter(|fix| turnpoint.inner().is_inside(fix))
                                .map(|f| Rc::clone(f))
                                .collect::<Vec<Rc<Fix>>>()
                        }
                    }).collect::<Vec<Vec<Rc<Fix>>>>();
                    todo!("make legs from the inside turnpoints precedence")

                }
                TaskType::AAT(_) => { todo!() }
            }
            todo!()
        }

        todo!()
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
}

enum TaskPiece {
    EntireTask,
    Leg(usize),
}