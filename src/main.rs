use std::fs::File;
use std::io::Read;
use std::time;

mod file_handling;

fn main() {
    println!("start");
    let start = time::Instant::now();
    let alts = file_handling::igc_parser::get_altitudes("examples/example.igc");
    for a in alts {
        println!("\tAltitude: {a}");
    }
    println!("{} ms since start", start.elapsed().as_millis());
}
