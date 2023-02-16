use igc::util::Time;
use crate::file_handling::igc_parser::Turnpoint;

pub enum TaskComponent {
    StartLine { tp: Turnpoint, length: u16 },
    Tp {tp: Turnpoint, radius: u16 },
    FinishRing { tp: Turnpoint, radius: u16 },
}

impl TaskComponent {
    //TODO
}

pub enum TaskType {
    AAT {
        start: TaskComponent::StartLine,
        points: Vec<TaskComponent::Tp>,
        finish: TaskComponent::FinishRing,
        task_time: Time,
    },
    AST {
        start: TaskComponent::StartLine,
        points: Vec<TaskComponent::Tp>,
        finish: TaskComponent::FinishRing,
    }
}

impl TaskType {
    //TODO
}