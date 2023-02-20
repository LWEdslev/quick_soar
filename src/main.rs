use std::time;
use crate::parser::util;
use quick_soar::*;

fn main() {
    let start = time::Instant::now();
    let c_records = util::get_turnpoint_descriptions(util::get_contents("examples/ast.igc").unwrap().as_str());
    println!("{}", c_records.len());
    for c in c_records {
        println!("\t{}", c.to_string());
    }
    println!("{} ms since start", start.elapsed().as_millis());
}
