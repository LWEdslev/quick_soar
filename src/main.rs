use std::any::Any;
use std::time;
use igc::util::Time;

use quick_soar::*;
use quick_soar::parser::util::{Fix, get_fixes};

fn main() {
    let path = "examples/aat.igc";

    let start = time::Instant::now();
    let contents = parser::util::get_contents(&path).unwrap();
    let task = parser::task::Task::parse(&contents).unwrap();
    let pilot_info = parser::pilot_info::PilotInfo::parse(&contents);
    let fixes = get_fixes(&contents);
    let mut flight = analysis::segmenting::Flight::make(fixes.clone());
    flight.print_segments(pilot_info.time_zone as u8);
    println!("--------------------------------");
    let flight = flight.get_subflight(Time::from_hms(10,0,0), Time::from_hms(12,0,0));
    flight.print_segments(pilot_info.time_zone as u8);
    println!("T count: {}", flight.count_thermals());

    //let contents = parser::util::get_contents("examples/CX.igc").unwrap();
    //let fixes = parser::util::get_fixes(&contents);

    //println!("{}", flight.thermal_percentage());
    println!("{} ms since start", start.elapsed().as_millis());
}
