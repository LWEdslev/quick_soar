use std::any::Any;
use std::time;
use igc::util::Time;
use regex::Match;
use crate::web_handling;

use quick_soar::*;
use quick_soar::analysis::calculation;
use quick_soar::analysis::calculation::TaskPiece;
use quick_soar::parser::util::{Fix, get_fixes};
use quick_soar::web_handling::soaringspot;

#[tokio::main]
async fn main() {
    let url = String::from("https://www.soaringspot.com/en_gb/central-plateau-contest-2023-taupo-2023/results/open/task-4-on-2023-03-08/daily");
    //let url = String::from("https://www.soaringspot.com/en_gb/sun-air-cup-junior-dm-2018-svaeveflyvecenter-arnborg-2018/results/junior-dm/task-9-on-2018-08-05/daily");
    let spot = soaringspot::SoaringSpot::new(url).await.unwrap();

    for s in spot.get_download_links() {
        println!("{}", s.unwrap_or("No file".to_string()))
    }
}
