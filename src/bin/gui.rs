use std::hash::Hash;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fs, thread};
use std::time::Duration;
use iced::{Alignment, Application, Command, Element, executor, futures, Renderer, Subscription, subscription, Theme, window};
use iced::alignment::{Horizontal, Vertical};
use iced::futures::future::BoxFuture;
use iced::widget::{button, column, container, row, text, text_input};
use iced::widget::progress_bar;
use iced::Settings;
use iced_native::command::Action;
use igc::util::{Date, Time};
use quick_soar::{analysis, parser, PathStrategy};
use quick_soar::analysis::calculation::Calculation;
use quick_soar::web_handling::soaringspot;
use quick_soar::web_handling::soaringspot::SoaringSpot;
use quick_soar::analysis::util::Offsetable;
use quick_soar::parser::task::Task;
use quick_soar::parser::util::get_date;

type Kph = f32;
type FloatMeters = f32;



pub fn main() -> iced::Result {
    let icon = None;

    AppState::run(Settings {
        id: None,
        window: window::Settings {
            size: (400, 180),
            position: Default::default(),
            min_size: None,
            max_size: None,
            visible: true,
            resizable: false,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon,
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
    GetSoaringspot,
    Downloading(Frac),
    Analyzing(Frac),
    Finished,
}

#[derive(Clone, Debug)]
enum Message {
    UrlChanged(String),
    GotSoaringspot(Result<SoaringSpot, String>),
    Downloaded(Frac),
    PreAnalysis,
    Analyzed(Frac),
    PostAnalysis,
    OpenFile,
}

struct AppState {
    input: String,
    progress: ProgressState,
    error_state: ErrorState,
    soaringspot: Option<SoaringSpot>,
    links: Option<Vec<Option<String>>>,
    contents: Vec<String>,
    date: Option<Date>,
    start_times: Vec<Option<Time>>,
    speeds: Vec<Option<Kph>>,
    distances: Vec<Option<FloatMeters>>,
    calculations: Vec<Calculation>,
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
                links: None,
                contents: vec![],
                date: None,
                start_times: vec![],
                speeds: vec![],
                distances: vec![],
                calculations: vec![],
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "QuickSoar - Gliding Analysis Tool".to_string()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self.progress {
            ProgressState::NotStarted | ProgressState::Finished => Subscription::none(),
            ProgressState::GetSoaringspot => {
                let path = PathStrategy::new().get_path();
                fs::create_dir(&path).unwrap_or(());
                soaringspot::clear(&path);
                fs::create_dir(&path).unwrap();
                let url = self.input.clone();
                let closure = move |_unit: ()| {
                    let url = url.clone();
                    async move {
                        let spot = SoaringSpot::new(url.clone()).await.unwrap();
                        let links = spot.get_download_links();
                        println!("{:?}", links);
                        (Message::GotSoaringspot(spot, links), ())
                    }
                };
                return subscription::unfold(0, (), closure)
            }
            ProgressState::Downloading(Frac(downloaded, total)) => {
                let links = self.links.as_ref().unwrap();
                let curr_file = links[downloaded].clone();
                let path = PathStrategy::new().get_path();
                let closure = move |_| {
                    let path = path.clone();
                    let curr_file = curr_file.clone();
                    async move {
                        if downloaded == total - 1 {
                            thread::sleep(Duration::from_millis(1000));
                            return (Message::PreAnalysis, ())
                        }
                        match curr_file {
                            None => {
                                (Message::Downloaded(Frac(downloaded + 1, total)), ())
                            }
                            Some(link) => {
                                soaringspot::download(&link, &path, downloaded).await;
                                (Message::Downloaded(Frac(downloaded + 1, total)), ())
                            }
                        }
                    }
                };

                subscription::unfold(downloaded, (), closure)
            }
            ProgressState::Analyzing(Frac(analyzed, total)) => {
                println!("now analyzing");
                if analyzed == total {
                    let finish_analysis_closure = move |_| {
                        async move {
                            (Message::PostAnalysis, ())
                        }
                    };
                    return subscription::unfold(analyzed, (), finish_analysis_closure)
                }

                let inc_closure = move |_| {
                    async move {
                        (Message::Analyzed(Frac(analyzed + 1, total)), ())
                    }
                };

                subscription::unfold(analyzed + total, (), inc_closure)
            }
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {

        println!("{:?}", message);
        match message {
            Message::UrlChanged(new_url) => {
                self.input = new_url;
            }
            Message::StartAnalysis => {
                self.progress = ProgressState::GetSoaringspot;
            }
            Message::PostAnalysis => {
                self.progress = ProgressState::Finished;
            }
            Message::OpenFile => {
                println!("Open file");
            }
            Message::Downloaded(frac) => {
                self.progress = ProgressState::Downloading(frac);
                println!("self.progress = {:?}", self.progress);
            }
            Message::Analyzed(Frac(analyzed, total)) => {
                println!("{}", self.contents.len());
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
            }
            Message::GotSoaringspot(spot, links) => {
                self.soaringspot = Some(spot);
                self.progress = ProgressState::Downloading(Frac(0, links.len()));
                self.links = Some(links);
            }
            Message::PreAnalysis => {

                let mut paths: Vec<_> = fs::read_dir(&PathStrategy::new().get_path()).unwrap()
                    .map(|r| r.unwrap())
                    .collect();
                paths.sort_by_key(|dir| dir.path());
                let contents = paths.into_iter().map(|path| {
                    parser::util::get_contents(path.path().display().to_string().as_str()).unwrap()
                }
                ).collect::<Vec<String>>();
                let spot = self.soaringspot.as_ref().unwrap();
                assert!(!contents.is_empty());
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
                println!("contents are now {}", self.contents.len());
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let AppState { input, progress, .. } = self;

        let (progress_percentage, progress_text) = match progress {
            ProgressState::NotStarted => (0., "Enter URL and start analysis".to_string()),
            ProgressState::Downloading(frac) => (frac.to_percentage(), format!("Downloading: {}/{}", frac.0, frac.1)),
            ProgressState::Analyzing(frac) => (frac.to_percentage(), format!("Analyzing: {}/{}", frac.0, frac.1)),
            ProgressState::Finished => (100., "Now you can open the analysis".to_string()),
            ProgressState::GetSoaringspot => (0., "Contacting soaringspot".to_string()),
        };

        let mut input_field = text_input::TextInput::new("", input).size(16).on_input(|s| Message::UrlChanged(s));

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



#[derive(Debug)]
enum DownloadState {
    NotStarted,
    Started,
    Finished,
}

#[derive(Debug)]
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

