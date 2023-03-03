use std::any::Any;
use std::time;
use igc::util::Time;

use quick_soar::*;
use quick_soar::analysis::calculation;
use quick_soar::analysis::calculation::TaskPiece;
use quick_soar::parser::util::{Fix, get_fixes};

fn main() {
    let path = "examples/aat.igc";

    let start = time::Instant::now();
    let contents = parser::util::get_contents(&path).unwrap();
    let task = parser::task::Task::parse(&contents).unwrap();
    let pilot_info = parser::pilot_info::PilotInfo::parse(&contents);
    let fixes = get_fixes(&contents);
    let mut flight = analysis::segmenting::Flight::make(fixes.clone());
    let start_time = Time::from_hms(10, 24, 30).seconds_since_midnight();
    let calculation = calculation::Calculation::new(task, flight, pilot_info, Some(start_time));

    let speed = calculation.speed(TaskPiece::EntireTask);
    if speed.is_some() {println!("Task Speed: {} km/h", speed.unwrap())};

    let legs_size = calculation.legs.len();
    for leg_index in 0..legs_size {
        let speed = calculation.speed(TaskPiece::Leg(leg_index));
        if speed.is_some() {print!("Leg {} Speed: {} km/h -- ", leg_index + 1, speed.unwrap())};
    }
    println!();

    let glide_ratio = calculation.glide_ratio(TaskPiece::EntireTask);
    if glide_ratio.is_some() {println!("Task Glide ratio: {}", glide_ratio.unwrap())};

    let legs_size = calculation.legs.len();
    for leg_index in 0..legs_size {
        let glide_ratio = calculation.glide_ratio(TaskPiece::Leg(leg_index));
        if glide_ratio.is_some() {print!("Leg {} Glide ratio: {} -- ", leg_index + 1, glide_ratio.unwrap())};
    }
    println!();

    //println!("{}", flight.thermal_percentage());
    println!("{} ms since start", start.elapsed().as_millis());
}
