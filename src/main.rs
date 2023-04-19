use std::fs;
use quick_soar::analysis::calculation::Calculation;
use quick_soar::excel::file_writer;
use quick_soar::{analysis, parser, web_handling};
use web_handling::soaringspot;
use analysis::util::Offsetable;
use quick_soar::parser::util::get_date;

#[tokio::main]
async fn main() {

    let time = std::time::Instant::now();
    let url = String::from(
        "https://www.soaringspot.com/en_gb/arnborg-easter-cup-svaeveflyvecenter-arnborg-2023/results/multiclass/task-1-on-2023-04-06/daily"
    );
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

    let mut paths: Vec<_> = fs::read_dir("igc_files/").unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());

    let contents = paths.into_iter().map(|path| {
        parser::util::get_contents(path.path().display().to_string().as_str()).unwrap()
        }
    ).collect::<Vec<String>>();
    assert!(!contents.is_empty());
    let date = get_date(contents[0].as_str()).unwrap();
    let start_times = spot.get_start_times();
    let speeds = spot.get_speeds();
    let distances = spot.get_distances();

    println!("{} ms since start, before calcs", time.elapsed().as_millis());

    let calculations = contents.into_iter().zip(start_times).zip(speeds).zip(distances).filter_map(|(((content, start_time), speed), dist)| {
        let task = parser::task::Task::parse(&content).ok()?;
        let fixes = parser::util::get_fixes(&content);
        let flight = analysis::segmenting::Flight::make(fixes);
        let pilot_info = parser::pilot_info::PilotInfo::parse(&content);
        println!("{}", pilot_info.comp_id);
        let time_zone = pilot_info.time_zone;
        let start_time = match start_time { None => None, Some(mut time) => { time.offset(-time_zone); Some(time.seconds_since_midnight()) } };
        let calculation = Calculation::new(task, flight, pilot_info, start_time, speed, dist);
        Some(calculation)
    }).collect::<Vec<Calculation>>();
    soaringspot::clear(path);

    println!("Now writing file");

    let some_calc = calculations.first().unwrap();
    file_writer::make_excel_file("./analysis.xlsx", some_calc.get_task(), &calculations, date);

    println!("{} ms since start", time.elapsed().as_millis());
}
