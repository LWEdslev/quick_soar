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
    println!("{}", fixes.len());
    println!("{}", task.points.len());
    if let parser::task::TaskType::AST = task.task_type {
        println!("yup");
    };
    println!("{}", pilot_info.glider_type);
    println!("{} ms since start", start.elapsed().as_millis());
}
