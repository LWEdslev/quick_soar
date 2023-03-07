use std::any::Any;
use std::time;
use igc::util::Time;

use quick_soar::*;
use quick_soar::analysis::calculation;
use quick_soar::analysis::calculation::TaskPiece;
use quick_soar::parser::util::{Fix, get_fixes};

fn main() {
    let path = "examples/aat.igc";

    let start = time::Instant::now();
    let contents = parser::util::get_contents(&path).unwrap();
    let task = parser::task::Task::parse(&contents).unwrap();
    let pilot_info = parser::pilot_info::PilotInfo::parse(&contents);
    let fixes = get_fixes(&contents);
    let mut flight = analysis::segmenting::Flight::make(fixes.clone());
    let start_time = Time::from_hms(10, 24, 30).seconds_since_midnight();
    let calculation = calculation::Calculation::new(task, flight, pilot_info, Some(start_time));

    let speed = calculation.speed(TaskPiece::EntireTask);
    if speed.is_some() {println!("Task Speed: {} km/h", speed.unwrap())};

    let legs_size = calculation.legs.len();
    for leg_index in 0..legs_size {
        let speed = calculation.speed(TaskPiece::Leg(leg_index));
        if speed.is_some() {print!("Leg {} Speed: {} km/h -- ", leg_index + 1, speed.unwrap())};
    }
    println!();

    let glide_ratio = calculation.glide_ratio(TaskPiece::EntireTask);
    if glide_ratio.is_some() {println!("Task Glide ratio: {}", glide_ratio.unwrap())};

    for leg_index in 0..legs_size {
        let glide_ratio = calculation.glide_ratio(TaskPiece::Leg(leg_index));
        if glide_ratio.is_some() {print!("Leg {} Glide ratio: {} -- ", leg_index + 1, glide_ratio.unwrap())};
    }
    println!();

    let start_time = calculation.start_time(TaskPiece::EntireTask);
    let start_time = start_time.as_ref();
    if start_time.is_some() {println!("Task Start time: {}:{}:{}", start_time.unwrap().hours, start_time.unwrap().minutes, start_time.unwrap().seconds)};

    for leg_index in 0..legs_size {
        let start_time = calculation.start_time(TaskPiece::Leg(leg_index));
        let start_time = start_time.as_ref();

        if start_time.is_some() {print!("Leg {} Start time: {}:{}:{} -- ", leg_index+1, start_time.unwrap().hours, start_time.unwrap().minutes, start_time.unwrap().seconds)};
    }
    println!();

    let thermal_percentage = calculation.climb_percentage(TaskPiece::EntireTask);
    if thermal_percentage.is_some() {println!("Task Glide ratio: {}", thermal_percentage.unwrap())};

    for leg_index in 0..legs_size {
        let thermal_percentage = calculation.climb_percentage(TaskPiece::Leg(leg_index));
        if thermal_percentage.is_some() {print!("Leg {} Climb%: {} -- ", leg_index + 1, thermal_percentage.unwrap())};
    }
    println!();

    let distance = calculation.distance(TaskPiece::EntireTask);
    if distance.is_some() {println!("Task Distance: {} km", distance.unwrap()/1000.)};

    for leg_index in 0..legs_size {
        let distance = calculation.distance(TaskPiece::Leg(leg_index));
        if distance.is_some() {print!("Leg {} distance: {} km -- ", leg_index + 1, distance.unwrap()/1000.)};
    }
    println!();

    let speed = calculation.climb_ground_speed(TaskPiece::EntireTask);
    if speed.is_some() {println!("Climb speed: {} kph", speed.unwrap())};

    for leg_index in 0..legs_size {
        let speed = calculation.climb_ground_speed(TaskPiece::Leg(leg_index));
        if speed.is_some() {print!("Leg {} climb speed: {} kph -- ", leg_index + 1, speed.unwrap())};
    }
    println!();

    let speed = calculation.glide_speed(TaskPiece::EntireTask);
    if speed.is_some() {println!("Glide speed: {} kph", speed.unwrap())};

    for leg_index in 0..legs_size {
        let speed = calculation.glide_speed(TaskPiece::Leg(leg_index));
        if speed.is_some() {print!("Leg {} glide speed: {} kph -- ", leg_index + 1, speed.unwrap())};
    }
    println!();

    let alt = calculation.start_alt(TaskPiece::EntireTask);
    if alt.is_some() {println!("Start altitude: {} m", alt.unwrap())};

    for leg_index in 0..legs_size {
        let alt = calculation.start_alt(TaskPiece::Leg(leg_index));
        if alt.is_some() {print!("Leg {} start altitude: {} m -- ", leg_index + 1, alt.unwrap())};
    }
    println!();

    let climb_rate = calculation.climb_rate(TaskPiece::EntireTask);
    if climb_rate.is_some() {println!("Task climb rate: {} mps", climb_rate.unwrap())};

    for leg_index in 0..legs_size {
        let climb_rate = calculation.climb_rate(TaskPiece::Leg(leg_index));
        if climb_rate.is_some() {print!("Leg {} climb rate: {} mps -- ", leg_index + 1, climb_rate.unwrap())};
    }
    println!();

    let percentage = calculation.excess_distance(TaskPiece::EntireTask);
    if percentage.is_some() {println!("Task excess distance: {}%", percentage.unwrap().floor() as u8)};

    for leg_index in 0..legs_size {
        let percentage = calculation.excess_distance(TaskPiece::Leg(leg_index));
        if percentage.is_some() {print!("Leg {} excess distance: {}% -- ", leg_index + 1, percentage.unwrap().floor() as u8)};
    }
    println!();

    println!("{} ms since start", start.elapsed().as_millis());
}
