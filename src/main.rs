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
    let start_time = Time::from_hms(10, 24, 00).seconds_since_midnight();
    let calculation = calculation::Calculation::new(task, flight, pilot_info, Some(start_time));

    let speed = calculation.speed(TaskPiece::EntireTask);
    if speed.is_some() {println!("Speed: {} km/h", speed.unwrap())};
    //let contents = parser::util::get_contents("examples/CX.igc").unwrap();
    //let fixes = parser::util::get_fixes(&contents);

    //println!("{}", flight.thermal_percentage());
    println!("{} ms since start", start.elapsed().as_millis());
}
