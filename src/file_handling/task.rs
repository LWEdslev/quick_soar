use std::error::Error;
use igc::util::Time;
use regex::Regex;
use crate::file_handling::igc_parser::TurnpointLocation;

enum DescriptionElem {
    R1, R2, A1, A2, Style,
}

impl DescriptionElem {
    fn get_element(&self, description: &str) -> Option<u16> {
        let (start, end) = match self {
            DescriptionElem::R1 => ("R1=", "m,"),
            DescriptionElem::R2 => ("R2=", "m"),
            DescriptionElem::A1 => ("A1=", ","),
            DescriptionElem::A2 => ("A2=", ","),
            DescriptionElem::Style => (",Style=", ","),
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
    fn parse(description: &str, loc: TurnpointLocation) -> Self {
        let style = DescriptionElem::Style.get_element(description);
        let tp = Turnpoint::parse(description, loc);
        match style {
            Some(2) => Self::Start(tp),//start
            Some(1) => Self::Tp(tp),
            Some(3) => Self::Finish(tp),
            None => panic!("style param not found in {description}"),
            _ => panic!("style was parsed to unkown number not in (1,2,3), {description}"),
        }
    }
}

pub struct Turnpoint {
    loc: TurnpointLocation,
    r1: u16,
    a1: u16,
    r2: u16,
    a2: u16,
}

impl Turnpoint {
    fn parse(description: &str, loc: TurnpointLocation) -> Self {
        let r1 = DescriptionElem::R1.get_element(description).unwrap();
        let a1 = DescriptionElem::A1.get_element(description).unwrap();
        let r2 = DescriptionElem::R2.get_element(description).unwrap();
        let a2 = DescriptionElem::A2.get_element(description).unwrap();
        Self {
            loc,
            r1,
            a1,
            r2,
            a2,
        }
    }
}


pub struct Task {
    points: Vec<TaskComponent>,
    task_type: TaskType,
}

enum TaskType {
    AAT(Time),
    AST,
}

impl Task {
    //TODO
}

#[cfg(test)]

mod tests {
    use crate::file_handling::igc_parser::get_turnpoints;
    use super::*;

    #[test]
    fn task_component_tp_parsing() {

        let mut turnpoint = get_turnpoints("LCU::C5624583N00924583E0005ViborgFlp");
        if let TaskComponent::Tp(comp) = TaskComponent::parse(
            "SEEYOU OZ=2,Style=1,SpeedStyle=1,R1=500m,A1=180,R2=0m,A2=0,MaxAlt=0.0m",
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
        let mut turnpoint = get_turnpoints("LCU::C5600500N00906683E0047FasterholtBanX");
        if let TaskComponent::Start(comp) = TaskComponent::parse(
            "SEEYOU OZ=-1,Style=2,SpeedStyle=0,R1=5000m,A1=180,R2=0m,A2=0,MaxAlt=0.0m,Line=1",
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
        let mut turnpoint = get_turnpoints("LCU::C5600633N00900867E0851ArnborgFlp");
        if let TaskComponent::Finish(comp) = TaskComponent::parse(
            "SEEYOU OZ=5,Style=3,SpeedStyle=2,R1=3000m,A1=180,R2=0m,A2=0,MaxAlt=0.0m,Reduce=1",
            turnpoint.remove(0)) {
            assert_eq!(comp.r1, 3000);
            assert_eq!(comp.a1, 180);
            assert_eq!(comp.r2, 0);
            assert_eq!(comp.a2, 0);
        } else {
            assert!(false);
        };
    }
}