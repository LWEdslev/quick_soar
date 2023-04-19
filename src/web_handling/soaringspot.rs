use std::fs::File;
use std::{fs, io};
use igc::util::Time;
use table_extract::Table;

type Kph = f32;

pub struct SoaringSpot {
    table: Table
}

impl SoaringSpot {
    pub async fn new(link: String) -> Result<Self, String> {
        let html = match reqwest::get(&link).await {
            Ok(html) => { match html.text().await {
                Ok(html) => html,
                Err(_) => return Err(format!("Unable to decode HTML body of {}", &link)),
            } },
            Err(_) => return Err(format!("Unable to access {}", &link)),
        };

        let table = match Table::find_first(&*html) {
            None => return Err(format!("No table found in {}", &link)),
            Some(table) => table,
        };
        Ok(Self { table })
    }

    pub fn get_download_links(&self) -> Vec<Option<String>> {
        let start = "<a href=&quot;";
        let end = "&quot;>";
        let regex_string = start.to_owned() + ".*" + end;
        let regex = regex::Regex::new(regex_string.as_str()).unwrap();

        self.table.iter().map(|row| {
            row.get("CN").unwrap_or("no CN data").to_string()
        }).map(|s| {
            let match_found = regex.find(&*s)?;
            let s = match_found.as_str();
            let mut s = s[start.len() .. s.len() - end.len()].to_owned();
            if s.starts_with("/") { //convert non-archive files to a http format
                s.insert_str(0, "https://www.soaringspot.com")
            }
            Some(s)
        }).collect::<Vec<Option<String>>>()
    }

    pub fn get_start_times(&self) -> Vec<Option<Time>> {
        self.table.iter().map(|row| {
            let time_string = row.get("Start")?.trim().to_string().split(":").map(|s| s.to_string()).collect::<Vec<String>>();
            if time_string.len() != 3 { return None }
            let (h, m, s) = (time_string[0].parse().ok()?, time_string[1].parse().ok()?, time_string[2].parse().ok()?);
            Some(Time::from_hms(h, m, s))
        }).collect::<Vec<Option<Time>>>()
    }

    pub fn get_speeds(&self) -> Vec<Option<Kph>> {
        self.table.iter().map(|row| {
            let speed_string = row.get("Speed")?
                .trim().to_string().split("&").next()?
                .parse::<f32>().ok()?;
            Some(speed_string)
        }).collect::<Vec<Option<Kph>>>()
    }
}

pub fn clear(path: &str) {
    fs::remove_dir_all(path).unwrap_or(())
}

pub async fn download(link: &String, path: &str, index: usize) {
    let filename = format!("{:0>3}.igc", index + 1);
    let resp = reqwest::get(link).await.unwrap().bytes().await.unwrap();
    let mut file = File::create(path.to_owned() + &*filename).unwrap();
    io::copy(&mut resp.as_ref(), &mut file).expect("failed to copy content");
}

