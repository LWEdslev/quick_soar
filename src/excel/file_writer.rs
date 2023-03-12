use std::fs;
use crate::analysis::calculation::Calculation;
use crate::parser::pilot_info::PilotInfo;
use crate::parser::task::Task;
use umya_spreadsheet::*;

fn make_excel_file(path: &str, task: Task, data: Vec<(Option<u32>, Calculation, PilotInfo)>) {
    let path = std::path::Path::new("/analysis.xlsx");
    match fs::remove_file(path) { Ok(_) => {} , Err(_) => {} }; //remove if present
    let mut book = reader::xlsx::read(path).unwrap();
    let ws = book.new_sheet("This is a sheet");

}