use std::fs;
use igc::util::Time;
use quick_soar::{analysis, parser};
use quick_soar::analysis::calculation::TaskPiece;
use quick_soar::web_handling::soaringspot;
use quick_soar::analysis::util::Offsetable;


#[tokio::main]
async fn main() {
    let time = std::time::Instant::now();
    println!("starting now");
    //let url = String::from("https://www.soaringspot.com/en_gb/central-plateau-contest-2023-taupo-2023/results/open/task-4-on-2023-03-08/daily");
    let url = String::from("https://www.soaringspot.com/en_gb/sun-air-cup-junior-dm-2018-svaeveflyvecenter-arnborg-2018/results/junior-dm/task-9-on-2018-08-05/daily");
    let spot = soaringspot::SoaringSpot::new(url).await.unwrap();

    let path = "igc_files/";
    soaringspot::clear(path);
    fs::create_dir(path).unwrap();
    for (index, link) in spot.get_download_links().iter().enumerate() {
        if let Some(link) = link {
            soaringspot::download(link, path, index).await;
            println!("Downloaded file {index}")
        } else {
            println!("No file for {index}")
        }
    }

    println!("{} ms since start", time.elapsed().as_millis());

    let paths = fs::read_dir("igc_files/").unwrap();

    let contents = paths.into_iter().map(|path|
        parser::util::get_contents(path.unwrap().path().display().to_string().as_str()).unwrap()
    );

    let start_times = spot.get_start_times();

    println!("{} ms since start, before calcs", time.elapsed().as_millis());

    let calculations = contents.zip(start_times).map(|(content, start_time)| {
        let task = parser::task::Task::parse(&content).ok()?;
        let fixes = parser::util::get_fixes(&content);
        let flight = analysis::segmenting::Flight::make(fixes);
        let pilot_info = parser::pilot_info::PilotInfo::parse(&content);
        let time_zone = pilot_info.time_zone;
        let start_time = match start_time { None => None, Some(mut time) => { time.offset(-time_zone); Some(time.seconds_since_midnight()) } };
        let calculation = analysis::calculation::Calculation::new(task, flight, pilot_info, start_time);
        println!("{}", calculation.climb_ground_speed(TaskPiece::EntireTask).unwrap_or(0.));
        println!("{}", calculation.climb_ground_speed(TaskPiece::Leg(0)).unwrap_or(0.));
        println!("{}", calculation.climb_ground_speed(TaskPiece::Leg(1)).unwrap_or(0.));
        println!("{}", calculation.glide_speed(TaskPiece::EntireTask).unwrap_or(0.));
        println!("{}", calculation.glide_speed(TaskPiece::Leg(0)).unwrap_or(0.));
        println!("{}", calculation.glide_speed(TaskPiece::Leg(1)).unwrap_or(0.));
        println!("{}", calculation.excess_distance(TaskPiece::EntireTask).unwrap_or(0.));
        println!("{}", calculation.excess_distance(TaskPiece::Leg(0)).unwrap_or(0.));
        println!("{}", calculation.excess_distance(TaskPiece::Leg(1)).unwrap_or(0.));
        Some(calculation)
    });
    println!("{}", calculations.last().unwrap().unwrap().total_flight.thermal_percentage());
    soaringspot::clear(path);
    println!("{} ms since start, after calcs", time.elapsed().as_millis());
}
