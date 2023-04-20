use std::collections::HashMap;
use std::fs;
use crate::analysis::calculation::{Calculation, TaskPiece};
use crate::parser::task::Task;
use umya_spreadsheet::*;
use crate::analysis::util::Offsetable;
use enum_iterator::{all, Sequence};
use igc::util::Date;
use umya_spreadsheet::helper::coordinate::CellCoordinates;

const GOOD_COLOR: &str = "FFCCFFCC";
const BAD_COLOR: &str = "FFFF99CC";

pub fn make_excel_file(path: &str, task: &Task, data: &Vec<Calculation>, date: Date) {
    let path = std::path::Path::new(path);
    fs::remove_file(path).unwrap_or(()); //remove if present
    let mut book = new_file();

    let entire_flight = book.new_sheet("Entire flight").unwrap();
    add_non_data_formatting(entire_flight, format!("{}-{}-{}", date.day, date.month, date.year).as_str(), TaskPiece::EntireTask);
    task.points.windows(2).enumerate().for_each(|(index, _)| {
        let ws = book.new_sheet("Placeholder").unwrap();
        add_non_data_formatting(ws, "DDMMYY", TaskPiece::Leg(index + 1));

    });

    let columns = all::<ColumnHeader>().collect::<Vec<ColumnHeader>>();

    let formatted_data = format_data(data, TaskPiece::EntireTask);

    for (index, column) in columns.iter().enumerate() {
        let coord = CellCoordinates { row: 2, col: (index + 1) as u32 };
        add_column_to_worksheet(book.get_sheet_mut(&1).unwrap(), column, formatted_data.get(column).unwrap(), coord);
        book.get_sheet_mut(&1).unwrap().get_row_dimension_mut(&2).set_height(120.);
    }

    book.remove_sheet(0).unwrap_or(()); //removes sheet that is created when the book is created

    for (index, _) in task.points.windows(2).enumerate() {
        let formatted_data = format_data(data, TaskPiece::Leg(index));
        let ws = book.get_sheet_mut(&(index+1)).unwrap();
        ws.get_row_dimension_mut(&2).set_height(120.);
        for (index, column) in columns.iter().enumerate() {
            let coord = CellCoordinates { row: 2, col: (index + 1) as u32 };
            add_column_to_worksheet(ws, column, formatted_data.get(column).unwrap(), coord);
        }
    }

    writer::xlsx::write(&book, path).unwrap();

}

fn add_non_data_formatting(worksheet: &mut Worksheet, date: &str, task_piece: TaskPiece) {
    let task_piece_string = match task_piece {
        TaskPiece::EntireTask => "Entire flight".to_string(),
        TaskPiece::Leg(i) => format!("Leg {}", i),
    };
    worksheet.set_name(task_piece_string.clone());
    let date_cell = worksheet.get_cell_mut("A1");
    date_cell.set_value_from_string(date);
    date_cell.get_style_mut().get_font_mut().set_name("Times New Roman").set_font_size(FontSize::default().set_val(10.).clone()).set_bold(true);
    date_cell.get_style_mut().get_alignment_mut().set_horizontal(HorizontalAlignmentValues::Center);
    let task_piece_cell = worksheet.get_cell_mut("B1");
    task_piece_cell.set_value_from_string(task_piece_string);
    task_piece_cell.get_style_mut().set_background_color_solid("FF9999FF").get_font_mut().set_name("Times New Roman").set_font_size(FontSize::default().set_val(10.).clone()).set_bold(true);
    task_piece_cell.get_style_mut().get_alignment_mut().set_horizontal(HorizontalAlignmentValues::Center);
    worksheet.add_merge_cells("B1:P1");
}

fn add_column_to_worksheet<T: Into<CellCoordinates>>(worksheet: &mut Worksheet, column: &ColumnHeader, data: &Vec<DataCell>, top_coord: T) {
    let top_coord = top_coord.into();
    let desc_cell = worksheet.get_cell_mut((top_coord.col, top_coord.row)).set_value_from_string(column.to_string());
    desc_cell.get_style_mut().get_alignment_mut().set_horizontal(HorizontalAlignmentValues::Center);
    desc_cell.get_style_mut().get_alignment_mut().set_text_rotation(90);
    desc_cell.get_style_mut().get_font_mut().set_name("Times New Roman").set_font_size(FontSize::default().set_val(10.).clone());
    let top_coord = CellCoordinates { row: top_coord.row + 1, col: top_coord.col };
    let unit_cell = worksheet.get_cell_mut((top_coord.col, top_coord.row));
        unit_cell.get_style_mut().get_font_mut().set_name("Times New Roman").set_font_size(FontSize::default().set_val(10.).clone()).set_bold(true);
        unit_cell.set_value_from_string(column.unit().unwrap_or(""));
        unit_cell.get_style_mut().get_alignment_mut().set_horizontal(HorizontalAlignmentValues::Center);
    let top_coord = CellCoordinates { row: top_coord.row + 2, col: top_coord.col }; //this moves down, and creates a gap
    for (index, d) in data.iter().enumerate() {
        let coord = CellCoordinates { row: top_coord.row + index as u32, col: top_coord.col };
        draw_data_cell_at(worksheet, d, coord);
    }
}

fn draw_data_cell_at<T: Into<CellCoordinates>>(worksheet: &mut Worksheet, cell: &DataCell, coord: T) {
    let extreme = &cell.extreme;
    let cell_value = &cell.value;
    let cell = match cell_value {
        CellValue::Float(val) => {
            let cell = worksheet.get_cell_mut(coord).set_value_number(*val);
            cell.get_style_mut().get_numbering_format_mut().set_format_code(NumberingFormat::FORMAT_NUMBER_00);
            cell.get_style_mut().get_alignment_mut().set_horizontal(HorizontalAlignmentValues::Center);
            cell
        }
        CellValue::Int(val) => {
            let cell = worksheet.get_cell_mut(coord).set_value_number(*val as f64);
            cell.get_style_mut().get_alignment_mut().set_horizontal(HorizontalAlignmentValues::Center);
            cell
        }
        CellValue::String(s) => {
            let cell = worksheet.get_cell_mut(coord).set_value_from_string(s);
            cell.get_style_mut().get_alignment_mut().set_horizontal(HorizontalAlignmentValues::Center);
            cell
        }
        CellValue::None => {
            let cell = worksheet.get_cell_mut(coord).set_value_from_string("---");
            cell.get_style_mut().get_alignment_mut().set_horizontal(HorizontalAlignmentValues::Center);
            cell
        }
    };

    cell.get_style_mut().get_font_mut().set_name("Times New Roman").set_font_size(FontSize::default().set_val(10.).clone());

    match extreme {
        Extreme::Best => { cell.get_style_mut().set_background_color(GOOD_COLOR).get_font_mut().set_bold(true); },
        Extreme::Worst => { cell.get_style_mut().set_background_color(BAD_COLOR).get_font_mut().set_bold(true); },
        Extreme::None => {},
    }



}

#[derive(Debug, PartialEq, Sequence, Hash, Eq, Copy, Clone)]
enum ColumnHeader {
    Ranking,
    Airplane,
    Callsign,
    Distance,
    StartTime,
    FinishTime,
    StartAlt,
    ClimbSpeed,
    ClimbRate,
    CruiseSpeed,
    GlideRatio,
    CruiseDistance,
    ExcessDistance,
    Speed,
    TurningPercentage,
    ThermalAltLoss,
    PercentBelow500,
    ThermalDrift,
}

impl ColumnHeader {
    fn to_string(&self) -> &str {
        use ColumnHeader::*;
        match self {
            StartTime => "Start time (Local)",
            StartAlt => "Start altitude (MSL)",
            Ranking => "Ranking",
            Airplane => "Airplane",
            Callsign => "Callsign",
            Distance => "Distance flown",
            FinishTime => "Finish time (Local)",
            ClimbRate => "Average rate of climb",
            ClimbSpeed => "Average climb speed",
            CruiseSpeed => "Average cruise speed",
            CruiseDistance => "Average glide distance",
            GlideRatio => "Average glide ratio",
            ExcessDistance => "Excess distance covered",
            Speed => "XC Speed",
            TurningPercentage => "Circling percentage",
            ThermalAltLoss => "Thermal altitude loss",
            PercentBelow500 => "Percentage below 500 QFE",
            ThermalDrift => "Task flown by thermal drifting"
        }
    }

    fn unit(&self) -> Option<&str> {
        use ColumnHeader::*;
        match self {
            Ranking | Airplane | Callsign | StartTime | FinishTime | GlideRatio => None,
            Distance => Some("[km]"),
            StartAlt => Some("[m]"),
            ClimbRate => Some("[m/s]"),
            CruiseSpeed | Speed | ClimbSpeed => Some("[km/h]"),
            CruiseDistance => Some("[km]"),
            ExcessDistance | ThermalAltLoss | TurningPercentage | PercentBelow500 | ThermalDrift => Some("[%]"),
        }
    }

    fn colorizable(&self) -> bool {
        use ColumnHeader::*;
        match self {
            Ranking | Airplane  | Callsign | Distance | StartTime | FinishTime => false,
            StartAlt | ClimbRate | ClimbSpeed | CruiseSpeed | CruiseDistance | GlideRatio
                | ExcessDistance | Speed | TurningPercentage | ThermalAltLoss | PercentBelow500 | ThermalDrift => true,
        }
    }

    fn is_highest_best_or_worst(&self) -> Extreme {
        use ColumnHeader::*;
        use Extreme::*;
        match self {
            StartAlt | ClimbRate | CruiseSpeed | CruiseDistance | GlideRatio | Speed | ThermalDrift => Best,
            ExcessDistance | TurningPercentage | ClimbSpeed | ThermalAltLoss | PercentBelow500 => Worst,
            _ => None,
        }
    }

    fn get_data_cells(&self, data: &Vec<Calculation>, task_piece: &TaskPiece) -> Vec<DataCell> {
        let task_piece = *task_piece;
        use ColumnHeader::*;
        let values = match self {
            Ranking => {
                (1 ..= data.len()).map(|i| CellValue::Int(i as i16)).collect::<Vec<CellValue>>()
            }
            Airplane => {
                data.iter().map(|d| {
                    let pilot_info = &d.pilot_info;
                    CellValue::String(pilot_info.glider_type.clone())
                }).collect::<Vec<CellValue>>()
            }
            Callsign => {
                data.iter().map(|d| {
                    let pilot_info = &d.pilot_info;
                    CellValue::String(pilot_info.comp_id.clone())
                }).collect::<Vec<CellValue>>()
            }
            Distance => {
                data.iter().map(|d| {
                    let calc = &d;
                    let dist = calc.distance(task_piece);
                    match dist {
                        None => CellValue::None,
                        Some(dist) => CellValue::Float(dist / 1000.)
                    }
                }).collect::<Vec<CellValue>>()
            }
            StartTime => {
                data.iter().map(|d| {
                    let calc = &d;
                    let start_time = calc.start_time(task_piece);
                    let utc_offset = d.pilot_info.time_zone;
                    match start_time {
                        None => CellValue::None,
                        Some(mut start_time) => {
                            start_time.offset(utc_offset);
                            CellValue::String(format!("{}:{}:{}", start_time.hours, start_time.minutes, start_time.seconds))
                        }
                    }
                }).collect::<Vec<CellValue>>()
            }
            FinishTime => {
                data.iter().map(|d| {
                    let calc = &d;
                    let finish_time = calc.finish_time(task_piece);
                    let utc_offset = d.pilot_info.time_zone;
                    match finish_time {
                        None => CellValue::None,
                        Some(mut finish_time) => {
                            finish_time.offset(utc_offset);
                            CellValue::String(format!("{}:{}:{}", finish_time.hours, finish_time.minutes, finish_time.seconds))
                        }
                    }
                }).collect::<Vec<CellValue>>()
            }
            StartAlt => {
                data.iter().map(|d| {
                    let calc = &d;
                    let alt = calc.start_alt(task_piece);
                    match alt {
                        None => CellValue::None,
                        Some(alt) => CellValue::Int(alt)
                    }
                }).collect::<Vec<CellValue>>()
            }
            ClimbRate => {
                data.iter().map(|d| {
                    let calc = &d;
                    let climb_rate = calc.climb_rate(task_piece);
                    match climb_rate {
                        None => CellValue::None,
                        Some(climb_rate) => CellValue::Float(climb_rate)
                    }
                }).collect::<Vec<CellValue>>()
            }
            ClimbSpeed => {
                data.iter().map(|d| {
                    let calc = &d;
                    let climb_speed = calc.climb_ground_speed(task_piece);
                    match climb_speed {
                        None => CellValue::None,
                        Some(climb_rate) => CellValue::Float(climb_rate)
                    }
                }).collect::<Vec<CellValue>>()
            }
            CruiseSpeed => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.glide_speed(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            CruiseDistance => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.glide_distance(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value / 1000.)
                    }
                }).collect::<Vec<CellValue>>()
            }
            GlideRatio => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.glide_ratio(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            ExcessDistance => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.excess_distance(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            Speed => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.speed(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            TurningPercentage => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.climb_percentage(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            ThermalAltLoss => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.thermal_height_loss(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            PercentBelow500 => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.time_below_500m_qfe(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            },
            ThermalDrift => {
                data.iter().map(|d| {
                    let calc = &d;
                    let value = calc.wind_thermal_gain(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
        };

        let colors = if self.colorizable() {
            use Extreme::*;
            let (high_val, low_val) = match self.is_highest_best_or_worst() {
                Best => (Best, Worst),
                Worst => (Worst, Best),
                None => unreachable!()
            };

            let min_max_closure = |x: &&CellValue ,y: &&CellValue| match (x,y) {
                (CellValue::Float(x_f), CellValue::Float(y_f)) => x_f.total_cmp(y_f),
                (CellValue::Int(x_i), CellValue::Int(y_i)) => x_i.cmp(y_i),
                _ => unreachable!(),
            };
            let max = values.iter()
                .filter(|v| match v {
                    CellValue::Float(_) | CellValue::Int(_) => true,
                    _ => false,
                })
                .max_by(min_max_closure);

            let min = values.iter()
                .filter(|v| match v {
                    CellValue::Float(_) | CellValue::Int(_) => true,
                    _ => false,
                })
                .min_by(min_max_closure);

            values.iter().map(|v| {
                let is_max = max.is_some() && v.is_numerically_equal_to(max.unwrap());
                let is_min = min.is_some() && v.is_numerically_equal_to(min.unwrap());
                if is_max {
                    high_val.clone()
                } else if is_min {
                    low_val.clone()
                } else {
                    None
                }
            }).collect::<Vec<Extreme>>()
        } else {
            values.iter().map(|_| Extreme::None).collect::<Vec<Extreme>>()
        };

        values.iter().zip(colors).map(|(v,c)| {
            DataCell::new(c, v.clone())
        }).collect::<Vec<DataCell>>()
    }
}

#[derive(Clone)]
enum CellValue {
    Float(f32),
    Int(i16),
    String(String),
    None,
}

impl CellValue {
    fn is_numerically_equal_to(&self, to: &CellValue) -> bool {
        match (self, to) {
            (CellValue::Float(l), CellValue::Float(r)) => {l==r}
            (CellValue::Int(l), CellValue::Int(r)) => {l==r}
            _ => false,
        }
    }
}

#[derive(Clone)]
enum Extreme { Best, Worst, None }

struct DataCell {
    extreme: Extreme,
    value: CellValue,
}

impl DataCell {
    fn new(extreme: Extreme, value: CellValue) -> Self {
        Self {
            extreme,
            value,
        }
    }
}

fn format_data(data: &Vec<Calculation>, task_piece: TaskPiece) -> HashMap<ColumnHeader, Vec<DataCell>> {
    let columns = all::<ColumnHeader>().collect::<Vec<ColumnHeader>>();
    let mut map = HashMap::new();
    for column in columns {
        map.insert(column, column.get_data_cells(data, &task_piece));
    };
    map
}