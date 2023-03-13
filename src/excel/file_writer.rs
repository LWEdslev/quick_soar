use std::collections::HashMap;
use std::fs;
use crate::analysis::calculation::{Calculation, TaskPiece};
use crate::parser::pilot_info::PilotInfo;
use crate::parser::task::Task;
use umya_spreadsheet::*;
use crate::analysis::util::Offsetable;

fn make_excel_file(path: &str, task: Task, data: Vec<(Option<u32>, Calculation, PilotInfo)>) {
    let path = std::path::Path::new("/analysis.xlsx");
    match fs::remove_file(path) { Ok(_) => {} , Err(_) => {} }; //remove if present
    let mut book = reader::xlsx::read(path).unwrap();
    let ws = book.new_sheet("This is a sheet");

}

enum ColumnHeader {
    Ranking,
    Airplane,
    Callsign,
    Distance,
    StartTime,
    FinishTime,
    StartAlt,
    ClimbRate,
    CruiseSpeed,
    CruiseDistance,
    GlideRatio,
    ExcessDistance,
    Speed,
    TurningPercentage,
    ThermalAltLoss,
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
            CruiseSpeed => "Average cruise speed",
            CruiseDistance => "Average glide distance",
            GlideRatio => "Average glide ratio",
            ExcessDistance => "Excess distance covered",
            Speed => "XC Speed",
            TurningPercentage => "Circling percentage",
            ThermalAltLoss => "Thermal altitude loss",
        }
    }

    fn unit(&self) -> Option<&str> {
        use ColumnHeader::*;
        match self {
            Ranking | Airplane | Callsign | StartTime | FinishTime | GlideRatio => None,
            Distance => Some("km"),
            StartAlt => Some("m"),
            ClimbRate => Some("m/s"),
            CruiseSpeed | Speed => Some("km/h"),
            CruiseDistance => Some("km"),
            ExcessDistance | ThermalAltLoss | TurningPercentage => Some("%"),
        }
    }

    fn colorizable(&self) -> bool {
        use ColumnHeader::*;
        match self {
            Ranking | Airplane | Callsign | Distance | StartTime | FinishTime => false,
            StartAlt | ClimbRate | CruiseSpeed | CruiseDistance | GlideRatio
                | ExcessDistance | Speed | TurningPercentage | ThermalAltLoss => true,
        }
    }

    fn is_highest_best_or_worst(&self) -> BestWorstNone {
        use ColumnHeader::*;
        use BestWorstNone::*;
        match self {
            StartAlt | ClimbRate | CruiseSpeed | CruiseDistance | GlideRatio | Speed => Best,
            ExcessDistance | TurningPercentage | ThermalAltLoss => Worst,
            _ => None,
        }
    }

    fn get_data_cells(&self, data: Vec<(Option<u32>, &Calculation, &PilotInfo)>, task_piece: TaskPiece) -> Vec<DataCell> {
        let task_piece = task_piece.clone();
        use ColumnHeader::*;
        let values = match self {
            Ranking => {
                (1 ..= data.len()).map(|i| CellValue::Int(i as i16)).collect::<Vec<CellValue>>()
            }
            Airplane => {
                data.iter().map(|d| {
                    let pilot_info = d.2;
                    CellValue::String(pilot_info.glider_type.clone())
                }).collect::<Vec<CellValue>>()
            }
            Callsign => {
                data.iter().map(|d| {
                    let pilot_info = d.2;
                    CellValue::String(pilot_info.comp_id.clone())
                }).collect::<Vec<CellValue>>()
            }
            Distance => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let dist = calc.distance(task_piece);
                    match dist {
                        None => CellValue::None,
                        Some(dist) => CellValue::Float(dist / 1000.)
                    }
                }).collect::<Vec<CellValue>>()
            }
            StartTime => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let start_time = calc.start_time(task_piece);
                    let utc_offset = d.2.time_zone;
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
                    let calc = d.1;
                    let finish_time = calc.finish_time(task_piece);
                    let utc_offset = d.2.time_zone;
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
                    let calc = d.1;
                    let alt = calc.start_alt(task_piece);
                    match alt {
                        None => CellValue::None,
                        Some(alt) => CellValue::Int(alt)
                    }
                }).collect::<Vec<CellValue>>()
            }
            ClimbRate => { todo!() }
            CruiseSpeed => { todo!() }
            CruiseDistance => { todo!() }
            GlideRatio => { todo!() }
            ExcessDistance => { todo!() }
            Speed => { todo!() }
            TurningPercentage => { todo!() }
            ThermalAltLoss => { todo!() }
        };
        todo!()
    }
}

enum CellValue {
    Float(f32),
    Int(i16),
    String(String),
    None,
}

enum BestWorstNone { Best, Worst, None }

struct DataCell {
    extreme: BestWorstNone,
    value: CellValue,
}

fn format_data(data: Vec<(Calculation, PilotInfo)>) -> HashMap<ColumnHeader, Vec<DataCell>> {
    todo!()
}