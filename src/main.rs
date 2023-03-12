use std::fs;
use std::fs::File;
use umya_spreadsheet::{CellStyle, CellValue, Color, reader, writer};

#[tokio::main]
async fn main() {
    let time = std::time::Instant::now();

    let path = std::path::Path::new("./analysis.xlsx");
    match fs::remove_file(path) { Ok(_) => {} , Err(_) => {} }; //remove if present
    let mut book = umya_spreadsheet::new_file();
    book.remove_sheet(0).unwrap();

    let sheet = book.new_sheet("Test sheet one").unwrap();
    let some_cell = sheet.get_cell_mut((1,1));
    some_cell.set_value_from_string("testytest");
    some_cell.get_style_mut().set_background_color(Color::COLOR_RED);
    sheet.get_row_dimension_mut(&1).set_height(200.);
    sheet.add_merge_cells("A1:C5");
    writer::xlsx::write(&book, path).unwrap();
    println!("{} ms since start", time.elapsed().as_millis());
}
