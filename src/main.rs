use std::time;
use crate::parser::igc_parser;

mod parser;

fn main() {
    let start = time::Instant::now();
    let c_records = igc_parser::get_turnpoint_descriptions(igc_parser::get_contents("examples/ast.igc").unwrap().as_str());
    println!("{}", c_records.len());
    for c in c_records {
        println!("\t{}", c.to_string());
    }
    println!("{} ms since start", start.elapsed().as_millis());
}
