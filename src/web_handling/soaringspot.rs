use igc::util::Time;
use table_extract::Table;

enum SoupError {
    IncorrectURL,
}

pub struct SoaringSpot {
    contents: String,
    table: Table
}

impl SoaringSpot {
    pub async fn new(link: String) -> Option<Self> {
        let html = reqwest::get(link).await.ok()?.text().await.ok()?;
        let table = Table::find_first(&*html)?;
        Some(Self { contents: html, table })
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
}

