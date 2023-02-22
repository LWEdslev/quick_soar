use std::any::Any;
use std::time;

use quick_soar::*;
use quick_soar::parser::util::get_fixes;

fn main() {
    let start = time::Instant::now();
    let contents = parser::util::get_contents("examples/ast.igc").unwrap();
    let task = parser::task::Task::parse(&contents).unwrap();
    let pilot_info = parser::pilot_info::PilotInfo::parse(&contents);
    let fixes = get_fixes(&contents);
    let flight = analysis::segmenting::Flight::make(fixes);
    //flight.print_segments();
    println!("{}", flight.thermal_percentage());
    println!("{} ms since start", start.elapsed().as_millis());
}
