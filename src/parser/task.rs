use igc::util::{Date, Time};
use regex::Regex;
use crate::parser::util;
use crate::parser::util::TurnpointRecord;

enum DescriptionElem {
    R1, R2, A1, A2, Style, AAT,
}

impl DescriptionElem {
    fn get_element(&self, description: &str) -> Option<u16> {
        let (start, end) = match self {
            DescriptionElem::R1 => ("R1=", "m,"),
            DescriptionElem::R2 => ("R2=", "m"),
            DescriptionElem::A1 => ("A1=", ","),
            DescriptionElem::A2 => ("A2=", ","),
            DescriptionElem::Style => (",Style=", ","),
            DescriptionElem::AAT => ("AAT=", ""),
        };

        let regex = Regex::new(format!("{start}[0-9]+{end}").as_str()).unwrap();
        let re_match = regex.find(&description);
        match re_match {
            None => None,
            Some(m) => Some(description[m.start()+start.len() .. m.end()-end.len()].parse().unwrap()),
        }
    }
}


pub enum TaskComponent {
    Tp(Turnpoint),
    Start(Turnpoint),
    Finish(Turnpoint),
}

impl TaskComponent {
    fn parse(description: &str, loc: TurnpointRecord) -> Self {
        let style = DescriptionElem::Style.get_element(description);
        let tp = Turnpoint::parse(description, loc);
        match style {
            Some(2) => Self::Start(tp),
            Some(1) => Self::Tp(tp),
            Some(3) => Self::Finish(tp),
            None => panic!("style param not found in {description}"),
            _ => panic!("style was parsed to unknown number not in (1,2,3), {description}"),
        }
    }

    fn is_aat(&self) -> bool {
        let tp = match self {
            TaskComponent::Tp(tp) => tp,
            TaskComponent::Start(tp) => tp,
            TaskComponent::Finish(tp) => tp,
        };
        tp.aat
    }

    pub(crate) fn inner(&self) -> &Turnpoint {
        match self {
            TaskComponent::Tp(inner) => inner,
            TaskComponent::Start(inner) => inner,
            TaskComponent::Finish(inner) => inner,
        }
    }
}

pub struct Turnpoint {
    pub latitude: f32,
    pub longitude: f32,
    pub name: Option<String>,
    pub r1: u16,
    pub a1: u16,
    pub r2: u16,
    pub a2: u16,
    aat: bool,
}

impl Turnpoint {
    pub(crate) fn parse(description: &str, loc: TurnpointRecord) -> Self {
        let r1 = DescriptionElem::R1.get_element(description).unwrap_or(0);
        let a1 = DescriptionElem::A1.get_element(description).unwrap_or(0);
        let r2 = DescriptionElem::R2.get_element(description).unwrap_or(0);
        let a2 = DescriptionElem::A2.get_element(description).unwrap_or(0);
        let aat = DescriptionElem::AAT.get_element(description).is_some();
        Self {
            latitude: loc.latitude,
            longitude: loc.longitude,
            name: loc.name,
            r1,
            a1,
            r2,
            a2,
            aat,
        }
    }
}


pub struct Task {
    pub points: Vec<TaskComponent>,
    pub task_type: TaskType,
}

pub enum TaskType {
    AAT(Time),
    AST,
}

#[derive(Debug)]
pub enum TaskError {
    NoStart,
    NoFinish,
    NoTurnpoints,
    NotSameAmountOfDescriptionsAsTurnpoints,
}

impl Task {
    pub fn parse(contents: &str) -> Result<Self, TaskError> {
        //the contents should be split into parts so there is not many unnecessary run through O(3n) -> O(n)
        let tps = util::get_turnpoint_locations(contents);
        let descriptions = util::get_turnpoint_descriptions(contents);
        let task_time = util::get_task_time(contents);
        if tps.len() != descriptions.len() { return Err(TaskError::NotSameAmountOfDescriptionsAsTurnpoints) };
        let points = tps.into_iter().zip(descriptions).map(|(tpl, desc)| {
            TaskComponent::parse(&*desc, tpl)
        }).collect::<Vec<TaskComponent>>();

        if points.len() < 3 { return Err(TaskError::NoTurnpoints) };

        match points.first() { //Checks if there is a turnpoint and if the first one is start
            Some(p) => match p {
                TaskComponent::Start(_) => {},
                _ => return Err(TaskError::NoStart)
            },
            None => return Err(TaskError::NoTurnpoints),
        }

        match points.last() { //checks if the last one is a finish
            Some(p) => match p {
                TaskComponent::Finish(_) => {},
                _ => return Err(TaskError::NoFinish)
            },
            None => return Err(TaskError::NoTurnpoints),
        }

        for i in &points[1..points.len()-2] { //checks if all points except first and last, are turnpoints
            match i {
                TaskComponent::Tp(_) => {}
                _ => return Err(TaskError::NoTurnpoints),
            }
        }

        let task_type = match points.get(1).unwrap().is_aat() {
            true => {
                match task_time {
                    None => TaskType::AST,
                    Some(time) => TaskType::AAT(time),
                }
            }
            false => TaskType::AST,
        };


        Ok(
            Self {
                points,
                task_type,
            }
        )
    }
}

#[cfg(test)]

mod tests {
    use crate::parser::util::get_turnpoint_locations;
    use super::*;

    #[test]
    fn task_component_tp_parsing() {

        let mut turnpoint = get_turnpoint_locations("LCU::C5624583N00924583E0005ViborgFlp");
        if let TaskComponent::Tp(comp) = TaskComponent::parse(
            "LSEEYOU OZ=2,Style=1,SpeedStyle=1,R1=500m,A1=180,R2=0m,A2=0,MaxAlt=0.0m",
            turnpoint.remove(0)) {
            assert_eq!(comp.r1, 500);
            assert_eq!(comp.a1, 180);
            assert_eq!(comp.r2, 0);
            assert_eq!(comp.a2, 0);
        } else {
            assert!(false);
        };
    }

    #[test]
    fn task_component_start_parsing() {
        let mut turnpoint = get_turnpoint_locations("LCU::C5600500N00906683E0047FasterholtBanX");
        if let TaskComponent::Start(comp) = TaskComponent::parse(
            "LSEEYOU OZ=-1,Style=2,SpeedStyle=0,R1=5000m,A1=180,R2=0m,A2=0,MaxAlt=0.0m,Line=1",
            turnpoint.remove(0)) {
            assert_eq!(comp.r1, 5000);
            assert_eq!(comp.a1, 180);
            assert_eq!(comp.r2, 0);
            assert_eq!(comp.a2, 0);
        } else {
            assert!(false);
        };
    }

    #[test]
    fn task_component_finish_parsing() {
        let mut turnpoint = get_turnpoint_locations("LCU::C5600633N00900867E0851ArnborgFlp");
        if let TaskComponent::Finish(comp) = TaskComponent::parse(
            "LSEEYOU OZ=5,Style=3,SpeedStyle=2,R1=3000m,A1=180,R2=0m,A2=0,MaxAlt=0.0m,Reduce=1",
            turnpoint.remove(0)) {
            assert_eq!(comp.r1, 3000);
            assert_eq!(comp.a1, 180);
            assert_eq!(comp.r2, 0);
            assert_eq!(comp.a2, 0);
        } else {
            assert!(false);
        };
    }

    #[test]
    fn ast_task_type_and_start_is_parsed_correctly() {
        let contents = util::get_contents("examples/ast.igc").unwrap();
        let task = Task::parse(&*contents).unwrap();
        let tps = task.points;
        match task.task_type {
            TaskType::AST => {},
            TaskType::AAT(_) => assert!(false),
        }
        if let Some(TaskComponent::Start(tp)) = tps.first() {
            if let Some(name) = &tp.name {
                assert_eq!(name.clone(), "0047FasterholtBanX".to_string())
            }
        } else {
            assert!(false)
        }
    }

    #[test]
    fn aat_task_type_and_start_is_parsed_correctly() {
        let contents = util::get_contents("examples/aat.igc").unwrap();
        let task = Task::parse(&*contents).unwrap();
        let tps = task.points;
        match task.task_type {
            TaskType::AAT(time) => assert_eq!(time, Time::from_hms(2, 0, 0)),
            TaskType::AST => assert!(false),
        }
        if let Some(TaskComponent::Start(tp)) = tps.first() {
            if let Some(name) = &tp.name {
                assert_eq!(name.clone(), "265Silas".to_string())
            }
        } else {
            assert!(false)
        }
    }
}