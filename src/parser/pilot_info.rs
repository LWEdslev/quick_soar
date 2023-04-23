use regex::Regex;

pub struct PilotInfo{
    pub glider_type: String,
    pub comp_id: String,
    pub time_zone: i8,
}

impl PilotInfo {
    pub fn parse(contents: &str) -> Self {
        let glider_type = PilotElem::GliderType.get_element(
            contents.lines().find(|s| s.trim().starts_with("LCU::HPGTYGLIDERTYPE:")).unwrap()
        );
        let comp_id = PilotElem::CompetitionId.get_element(
            contents.lines().find(|s| s.trim().starts_with("LCU::HPCIDCOMPETITIONID:")).unwrap()
        );
        let time_zone = PilotElem::TimeZone.get_element(
            contents.lines().find(|s| s.trim().starts_with("LCU::HPTZNTIMEZONE:")).unwrap()
        ).parse::<i8>().unwrap();

        Self {
            glider_type,
            comp_id,
            time_zone,
        }
    }
}

enum PilotElem {
    GliderType,
    CompetitionId,
    TimeZone,
}

impl PilotElem {
    fn get_element(&self, description: &str) -> String {
        let start = match self {
            PilotElem::GliderType => "LCU::HPGTYGLIDERTYPE:",
            PilotElem::CompetitionId => "LCU::HPCIDCOMPETITIONID:",
            PilotElem::TimeZone => "LCU::HPTZNTIMEZONE:",
        };

        let regex = Regex::new(format!("{start}.+").as_str()).unwrap();
        let m = regex.find(description.trim()).unwrap();

        (description.trim()[m.start()+start.len() .. m.end()]).to_string()
    }
}

#[cfg(test)]

mod tests {
    use crate::parser::util;
    use super::*;

    #[test]
    fn pilot_info_ast_parsing() {
        let contents = util::get_contents("examples/ast.igc").unwrap();
        let pilot_info = PilotInfo::parse(&contents);
        assert_eq!(pilot_info.comp_id, "KE");
        assert_eq!(pilot_info.time_zone, 2);
        assert_eq!(pilot_info.glider_type, "LS 8");

    }

    #[test]
    fn negative_time_zone() {
        let contents =
            "LCU::HPPLTPILOT:Kevin Kjær Andersen
            LCU::HPGTYGLIDERTYPE:LS 8
            LCU::HPGIDGLIDERID:OY-XXK
            LCU::HPCCLCOMPETITIONCLASS:
            LCU::HPCIDCOMPETITIONID:KE
            LCU::HPATS:102153
            LCU::HPELVELEVATION:30
            LCU::HPTZNTIMEZONE:-2";

        let pilot_info = PilotInfo::parse(contents);
        assert_eq!(pilot_info.time_zone, -2);
    }
}