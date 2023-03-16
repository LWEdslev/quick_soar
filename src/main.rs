use std::fs;
use quick_soar::analysis::calculation::Calculation;
use quick_soar::excel::file_writer;
use quick_soar::{analysis, parser, web_handling};
use web_handling::soaringspot;
use analysis::util::Offsetable;

#[tokio::main]
async fn main() {

    let time = std::time::Instant::now();
    let url = String::from(
        "https://www.soaringspot.com/en_gb/junior-world-gliding-championships-2022-tabor-2022/results/class-club/task-3-on-2022-08-02/daily"
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
    );


    let start_times = spot.get_start_times();

    println!("{} ms since start, before calcs", time.elapsed().as_millis());

    let calculations = contents.zip(start_times).filter_map(|(content, start_time)| {
        let task = parser::task::Task::parse(&content).ok()?;
        let fixes = parser::util::get_fixes(&content);
        let flight = analysis::segmenting::Flight::make(fixes);
        let pilot_info = parser::pilot_info::PilotInfo::parse(&content);
        println!("{}", pilot_info.comp_id);
        let time_zone = pilot_info.time_zone;
        let start_time = match start_time { None => None, Some(mut time) => { time.offset(-time_zone); Some(time.seconds_since_midnight()) } };
        let calculation = Calculation::new(task, flight, pilot_info, start_time);
        Some(calculation)
    }).collect::<Vec<Calculation>>();
    soaringspot::clear(path);

    println!("Now writing file");

    let some_calc = calculations.first().unwrap();
    file_writer::make_excel_file("./analysis.xlsx", some_calc.get_task(), &calculations);

    println!("{} ms since start", time.elapsed().as_millis());
}
