use std::{fs, io};
use quick_soar::analysis::calculation::Calculation;
use quick_soar::excel::file_writer;
use quick_soar::{analysis, parser, web_handling};
use web_handling::soaringspot;
use analysis::util::Offsetable;
use quick_soar::parser::util::get_date;
use colored::Colorize;

#[tokio::main]
async fn main() {
    println!("Enter URL:");
    let mut url = String::new();

    io::stdin()
        .read_line(&mut url)
        .expect("Failed to read line");
    let mut spot = soaringspot::SoaringSpot::new(url).await;

    while spot.is_err() {
        println!("Invalid URL, enter a new one:");
        let mut url = String::new();
        io::stdin()
        .read_line(&mut url)
        .expect("Failed to read line");
        spot = soaringspot::SoaringSpot::new(url).await;
    }
    let spot = spot.unwrap();

    let path_strategy = if cfg!(target_os = "windows") {
        PathStrategy::Windows
    } else if cfg!(target_os = "macos") {
        PathStrategy::MacOS
    } else if cfg!(target_os = "linux") {
        PathStrategy::Linux
    } else {
        panic!("unrecognized OS")
    };

    let path = path_strategy.get_path();

    fs::create_dir(&path).unwrap_or(());
    soaringspot::clear(&path);
    fs::create_dir(&path).unwrap();
    let links = spot.get_download_links();
    for (index, link) in links.iter().enumerate() {
        if let Some(link) = link {
            soaringspot::download(link, &path, index).await;
            println!("Downloaded file {}/{}", format!("{}", index + 1).blue().bold(), format!("{}", links.len()).blue().bold())
        } else {
            println!("No file for {}", index + 1)
        }
    }

    let mut paths: Vec<_> = fs::read_dir(&path).unwrap()
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

    let calculations = contents.into_iter().zip(start_times).zip(speeds).zip(distances).filter_map(|(((content, start_time), speed), dist)| {
        let task = parser::task::Task::parse(&content).ok()?;
        let fixes = parser::util::get_fixes(&content);
        let flight = analysis::segmenting::Flight::make(fixes);
        let pilot_info = parser::pilot_info::PilotInfo::parse(&content);
        println!("Analyzing: {}", pilot_info.comp_id.blue().bold());
        let time_zone = pilot_info.time_zone;
        let start_time = match start_time { None => None, Some(mut time) => { time.offset(-time_zone); Some(time.seconds_since_midnight()) } };
        let calculation = Calculation::new(task, flight, pilot_info, start_time, speed, dist);
        Some(calculation)
    }).collect::<Vec<Calculation>>();
    soaringspot::clear(&path);
    
    let some_calc = calculations.first().unwrap();
    let analysis_path = format!("{}analysis_{}_{}_{}.xlsx", path_strategy.get_path(), date.day, date.month, date.year);
    fs::create_dir(&path).unwrap();
    file_writer::make_excel_file(&analysis_path, some_calc.get_task(), &calculations, date);
    println!("Finished analysis. Opening file"); 
    opener::open(&analysis_path).unwrap();
}


enum PathStrategy {
    Linux,
    Windows,
    MacOS,
}

impl PathStrategy {
    fn get_path(&self) -> String {
        match self {
            PathStrategy::Linux => {
                match home::home_dir() {
                    Some(path) => path.to_str().unwrap().to_string() + &*"/.quicksoar/",
                    None => panic!("no home directory found"),
                }
            }
            PathStrategy::Windows => {
                match home::home_dir() {
                    Some(path) => path.to_str().unwrap().to_string() + &*"/.quicksoar/",
                    None => panic!("no home directory found"),
                }
            }
            PathStrategy::MacOS => {
                match home::home_dir() {
                    Some(path) => path.to_str().unwrap().to_string() + &*"/.quicksoar/",
                    None => panic!("no home directory found"),
                }
            }
        }
    }
}