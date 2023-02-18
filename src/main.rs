use std::time;
use crate::file_handling::igc_parser;

mod file_handling;

fn main() {
    let start = time::Instant::now();
    let c_records = igc_parser::get_turnpoints(igc_parser::get_contents("examples/example.igc").unwrap().as_str());
    println!("{}", c_records.len());
    for c in c_records {
        println!("\t{}", c.to_string());
    }
    println!("{} ms since start", start.elapsed().as_millis());
}
