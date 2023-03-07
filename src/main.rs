use std::any::Any;
use std::time;
use igc::util::Time;
use crate::web_handling;

use quick_soar::*;
use quick_soar::analysis::calculation;
use quick_soar::analysis::calculation::TaskPiece;
use quick_soar::parser::util::{Fix, get_fixes};

#[tokio::main]
async fn main() {
    let url = String::from("https://www.soaringspot.com/en_gb/central-plateau-contest-2023-taupo-2023/results/open/task-4-on-2023-03-08/daily");
    let soup = web_handling::soup::Soup::new(url).await.unwrap();
    println!("{}", soup.html);
}
