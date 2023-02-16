use std::time;
use crate::file_handling::igc_parser;

mod file_handling;

fn main() {
    println!("start");
    let start = time::Instant::now();
    let fixes = igc_parser::get_fixes(igc_parser::get_contents("examples/example.igc").unwrap());
    for fix in fixes {
        println!("\t{}", fix.to_string());
    }
    println!("{} ms since start", start.elapsed().as_millis());
}
