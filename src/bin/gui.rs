use std::hash::Hash;
use std::{fs, thread};
use std::path::Path;
use iced::{Alignment, Application, Command, Element, executor, Theme, window};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, text, text_input};
use iced::widget::progress_bar;
use iced::Settings;
use iced::window::{icon, Position};
use igc::util::{Date, Time};
use image::ImageFormat;
use quick_soar::{analysis, parser, PathStrategy};
use quick_soar::analysis::calculation::Calculation;
use quick_soar::web_handling::soaringspot;
use quick_soar::web_handling::soaringspot::SoaringSpot;
use quick_soar::analysis::util::Offsetable;
use quick_soar::excel::file_writer;
use quick_soar::parser::task::Task;
use quick_soar::parser::util::get_date;

type Kph = f32;
type FloatMeters = f32;

pub fn main() -> iced::Result {
    let bytes = include_bytes!("qsicon.png");
    let icon = icon::from_file_data(bytes, Some(ImageFormat::Png)).expect("unable to make icon");

    AppState::run(Settings {
        id: None,
        window: window::Settings {
            size: (400, 180),
            position: Position::Centered,
            min_size: None,
            max_size: None,
            visible: true,
            resizable: false,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: Some(icon),
            platform_specific: Default::default(),
        },
        flags: (),
        default_font: Default::default(),
        default_text_size: 30.,
        text_multithreading: true,
        antialiasing: true,
        exit_on_close_request: true,
        try_opengles_first: false,
    })
}

#[derive(PartialEq, Clone, Debug)]
enum ProgressState {
    NotStarted,
    Downloading(Frac),
    Analyzing(Frac),
    Finished,
    IncorrectURL,
}

#[derive(Clone, Debug)]
enum Message {
    UrlChanged(String),
    StartAnalysis, // Button
    GotSoaringspot(Result<SoaringSpot, String>),
    Downloading(Frac),
    PreAnalysis(Result<(), GUIError>),
    Analyzed(Frac),
    PostAnalysis(Result<(), GUIError>),
    OpenFile, // Button
}

struct AppState {
    input: String,
    progress: ProgressState,
    error_state: ErrorState,
    soaringspot: Option<SoaringSpot>,
    links: Vec<Option<String>>,
    contents: Vec<String>,
    date: Option<Date>,
    start_times: Vec<Option<Time>>,
    speeds: Vec<Option<Kph>>,
    distances: Vec<Option<FloatMeters>>,
    calculations: Vec<Calculation>,
    path: String,
    analysis_path: Option<String>,
}


#[derive(PartialEq, Clone, Debug)]
struct Frac(usize, usize);

impl Frac {
    fn to_percentage(&self) -> f32 {
        (self.0 as f32 / self.1 as f32) * 100.
    }
    fn is_max(&self) -> bool { self.0 == self.1 }
    fn increment(self) -> Self {
        Self(self.0 + 1, self.1)
    }
}

impl Application for AppState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                input: "".to_string(),
                progress: ProgressState::NotStarted,
                error_state: ErrorState::None,
                soaringspot: None,
                links: vec![],
                contents: vec![],
                date: None,
                start_times: vec![],
                speeds: vec![],
                distances: vec![],
                calculations: vec![],
                path: PathStrategy::new().get_path(),
                analysis_path: None,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "QuickSoar - Gliding Analysis Tool".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {

        println!("{:?}", message);
        match message {
            Message::UrlChanged(new_url) => {
                self.input = new_url;
                Command::none()
            }
            Message::StartAnalysis => {
                self.contents.clear();
                self.soaringspot = None;
                self.date = None;
                self.calculations.clear();
                self.distances.clear();
                self.speeds.clear();
                self.start_times.clear();
                self.links.clear();
                async fn contact_soaringspot(url: String) -> Result<SoaringSpot, String> {
                    let spot = SoaringSpot::new(url).await;
                    spot
                }
                Command::perform(contact_soaringspot(self.input.clone()), Message::GotSoaringspot)
            }
            Message::GotSoaringspot(Err(_)) => {
                self.progress = ProgressState::IncorrectURL;
                Command::none()
            }
            Message::GotSoaringspot(Ok(spot)) => {
                fs::create_dir(&self.path).unwrap_or(());
                soaringspot::delete_files_in_dir(&self.path);
                let links = spot.get_download_links();
                self.soaringspot = Some(spot);
                self.links = links;

                async fn start_download(length: usize) -> Frac {
                    Frac(0, length)
                }

                Command::perform(start_download(self.links.len()), Message::Downloading)
            }

            Message::Downloading(frac) if frac.is_max() => {
                self.progress = ProgressState::Downloading(frac);
                Command::perform(async {Ok(())}, Message::PreAnalysis)
            }

            Message::Downloading(frac) => {
                let link = self.links[frac.0].clone();
                let path = self.path.clone();
                self.progress = ProgressState::Downloading(frac.clone());
                async fn download(link: Option<String>, path: String, frac: Frac) -> Frac {
                    if let Some(link) = link { soaringspot::download(&link, &path, frac.0).await; }
                    frac.increment()
                }
                Command::perform(download(link, path, frac.clone()), Message::Downloading)
            }

            Message::PreAnalysis(_) => {
                let mut paths: Vec<_> = fs::read_dir(&PathStrategy::new().get_path()).unwrap()
                    .map(|r| r.unwrap())
                    .filter(|dir| dir.path().is_file())
                    .collect();
                paths.sort_by_key(|dir| dir.path());
                let contents = paths.into_iter().map(|path| {
                    parser::util::get_contents(path.path().display().to_string().as_str()).unwrap()
                }).collect::<Vec<String>>();

                async fn pre_analysis(length: usize) -> Frac {
                    Frac(0, length)
                }

                let spot = self.soaringspot.as_ref().unwrap();
                let date = get_date(contents[0].as_str()).unwrap();
                let start_times = spot.get_start_times();
                let speeds = spot.get_speeds();
                let distances = spot.get_distances();
                self.date = Some(date);
                self.start_times = start_times;
                self.speeds = speeds;
                self.distances = distances;
                self.contents = contents;
                self.progress = ProgressState::Analyzing(Frac(0, self.contents.len()));
                println!("length of contents is {}", self.contents.len());
                Command::perform(pre_analysis(self.contents.len()), Message::Analyzed)
            }

            Message::Analyzed(frac) if frac.is_max() => {
                self.progress = ProgressState::Analyzing(frac);
                Command::perform(async {Ok(())}, Message::PostAnalysis)
            }

            Message::Analyzed(Frac(analyzed, total)) => {
                let content = self.contents.get(analyzed).unwrap().clone();
                let speed = self.speeds.get(analyzed).unwrap().clone();
                let dist = self.distances.get(analyzed).unwrap().clone();
                let start_time = self.start_times.get(analyzed).unwrap().clone();
                let calc: Option<Calculation> = {
                    match Task::parse(&content).ok() {
                        None => None,
                        Some(task) => {
                            let fixes = parser::util::get_fixes(&content);
                            let flight = analysis::segmenting::Flight::make(fixes);
                            let pilot_info = parser::pilot_info::PilotInfo::parse(&content);
                            let time_zone = pilot_info.time_zone;

                            // I have to do stupid shit like this when you don't derive Clone in you APIs!!!
                            let start_time = match start_time {
                                None => None,
                                Some(time) => Some(Time::from_hms(time.hours, time.minutes, time.seconds)),
                            };
                            let start_time = match start_time { None => None, Some(mut time) => { time.offset(-time_zone); Some(time.seconds_since_midnight()) } };
                            let calculation = Calculation::new(task, flight, pilot_info, start_time, speed, dist);
                            Some(calculation)
                        },
                    }
                };
                if let Some(calc) = calc {self.calculations.push(calc)}
                self.progress = ProgressState::Analyzing(Frac(analyzed, total));
                Command::perform(async move { Frac(analyzed + 1, total) }, Message::Analyzed)
            }

            Message::PostAnalysis(_) => {
                self.progress = ProgressState::Finished;
                let some_calc = self.calculations.first().unwrap();
                let date = match self.date {
                    None => Date::from_dmy(0,0,0),
                    Some(Date { day, month, year }) => Date::from_dmy(day, month, year),
                };
                let class: Option<String> = {
                    let url = self.input.clone();
                    let mut parts = url.split("/").collect::<Vec<&str>>();
                    println!("partslen is {}", parts.len());
                    match parts.iter().position(|p| p.starts_with("results")) {
                        None => None,
                        Some(index) => {
                            let class = parts.get(index + 1).unwrap();
                            Some(class.to_string())
                        },
                    }
                };
                let analysis_path = format!("{}analysis/QS-{}-{}-{}-{}.xlsx", &self.path, class.unwrap_or("".to_string()), date.day, date.month, date.year);
                println!("analysis path is {}", analysis_path);
                let analysis_path = soaringspot::make_file_name_unique(analysis_path.as_str());
                println!("analysis path is {}", analysis_path);
                soaringspot::delete_files_in_dir(&self.path);
                fs::create_dir(format!("{}/analysis", &self.path)).unwrap_or(());
                file_writer::make_excel_file(&analysis_path, some_calc.get_task(), &self.calculations, date);
                self.analysis_path = Some(analysis_path);
                Command::none()
            }
            Message::OpenFile => {
                println!("Open file");
                opener::open(self.analysis_path.as_ref().unwrap()).unwrap();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let AppState { input, progress, .. } = self;

        let (progress_percentage, progress_text) = match progress {
            ProgressState::NotStarted => (0., "Enter URL and start analysis".to_string()),
            ProgressState::Downloading(frac) => (frac.to_percentage(), format!("Downloading: {}/{}", frac.0, frac.1)),
            ProgressState::Analyzing(frac) => (frac.to_percentage(), format!("Analyzing: {}/{}", frac.0, frac.1)),
            ProgressState::Finished => (100., "Now you can open the analysis".to_string()),
            ProgressState::IncorrectURL => (0., "URL must be from a SoaringSpot competition day".to_string()),
        };

        let input_field = text_input::TextInput::new("", input).size(16).on_input(|s| Message::UrlChanged(s));

        let txt = text::Text::new("Enter URL: ").size(20).vertical_alignment(Vertical::Center);
        let url_row = row![
            txt,
            input_field
        ].align_items(Alignment::Center).padding(10);

        let progress_text = row![
            text(progress_text).size(20).vertical_alignment(Vertical::Center)
        ].padding(10);

        let progress_row = row![
            progress_bar(0. ..= 100., progress_percentage).width(380).height(20)
        ].padding(10);

        let open_file_button = button(
            text("Open analysis")
                .size(20)
                .horizontal_alignment(Horizontal::Center)
        ).height(30).width(185);

        let open_file_button = if let ProgressState::Finished = progress {
            open_file_button.on_press(Message::OpenFile)
        } else {
            open_file_button
        };

        let button_row = row![
            button(
                text("Start analysis")
                    .size(20)
                    .horizontal_alignment(Horizontal::Center)
            )
                .on_press(Message::StartAnalysis)
                .height(30)
                .width(185),
            open_file_button
        ].spacing(10).padding(10);

        let col = column(vec![url_row.into(),
                              progress_text.into(),
                              progress_row.into(),
                              button_row.into()]);

        container(col).into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

}

pub enum State {
    Ready(String),
    Downloading {
        response: reqwest::Response,
        total: u64,
        downloaded: u64,
    },
    Finished,
}


enum FileProgress {
    NotStarted, Started, Finished, Errored
}

#[derive(Debug, Clone)]
enum GUIError {
    FailedDownloading,
    FailedWriting,
}

#[derive(Debug)]
enum ErrorState {
    None,
    Restartable(String),
    Fatal(String),
}

