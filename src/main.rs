use std::fs;
use quick_soar::parser;
use quick_soar::web_handling::soaringspot;

#[tokio::main]
async fn main() {
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

    let paths = fs::read_dir("igc_files/").unwrap();

    for path in paths {
        let contents = parser::util::get_contents(path.unwrap().path().display().to_string().as_str()).unwrap();
        println!("{}", contents.len())
    }
}
