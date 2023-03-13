use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use crate::analysis::calculation::{Calculation, TaskPiece};
use crate::parser::pilot_info::PilotInfo;
use crate::parser::task::Task;
use umya_spreadsheet::*;
use crate::analysis::util::Offsetable;
use crate::excel::file_writer::BestWorstNone::Best;

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
            ClimbRate => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let climb_rate = calc.climb_rate(task_piece);
                    match climb_rate {
                        None => CellValue::None,
                        Some(climb_rate) => CellValue::Float(climb_rate)
                    }
                }).collect::<Vec<CellValue>>()
            }
            CruiseSpeed => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let value = calc.glide_speed(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            CruiseDistance => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let value = calc.glide_distance(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            GlideRatio => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let value = calc.glide_ratio(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            ExcessDistance => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let value = calc.excess_distance(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            Speed => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let value = calc.speed(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            TurningPercentage => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let value = calc.climb_percentage(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
            ThermalAltLoss => {
                data.iter().map(|d| {
                    let calc = d.1;
                    let value = calc.thermal_height_loss(task_piece);
                    match value {
                        None => CellValue::None,
                        Some(value) => CellValue::Float(value)
                    }
                }).collect::<Vec<CellValue>>()
            }
        };

        let colors = if self.colorizable() {
            use BestWorstNone::*;
            let (high_val, low_val) = match self.is_highest_best_or_worst() {
                Best => (Best, Worst),
                Worst => (Worst, Best),
                None => unreachable!()
            };

            let min_max_closure = |x: &&CellValue ,y: &&CellValue| match (x,y) {
                (CellValue::Float(x_f), CellValue::Float(y_f)) => x_f.total_cmp(&y_f),
                (CellValue::Int(x_i), CellValue::Int(y_i)) => x_i.cmp(&y_i),
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
            }).collect::<Vec<BestWorstNone>>()
        } else {
            values.iter().map(|_| BestWorstNone::None).collect::<Vec<BestWorstNone>>()
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
enum BestWorstNone { Best, Worst, None }

struct DataCell {
    extreme: BestWorstNone,
    value: CellValue,
}

impl DataCell {
    fn new(extreme: BestWorstNone, value: CellValue) -> Self {
        Self {
            extreme,
            value,
        }
    }
}

fn format_data(data: Vec<(Calculation, PilotInfo)>) -> HashMap<ColumnHeader, Vec<DataCell>> {
    todo!()
}