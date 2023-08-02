use regex::Regex;

pub struct PilotInfo{
    pub glider_type: String,
    pub comp_id: String,
    pub time_zone: i8,
}

#[derive(Debug)]
pub struct PilotInfoParseError;

impl PilotInfo {
    pub fn parse(contents: &str) -> Result<Self, PilotInfoParseError> {
        let glider_type = PilotElem::GliderType.get_element(
            contents.lines().find(|s| s.trim().starts_with("LCU::HPGTYGLIDERTYPE:"))
        );
        let comp_id = PilotElem::CompetitionId.get_element(
            contents.lines().find(|s| s.trim().starts_with("LCU::HPCIDCOMPETITIONID:"))
        );
        let time_zone = PilotElem::TimeZone.get_element(
            contents.lines().find(|s| s.trim().starts_with("LCU::HPTZNTIMEZONE:"))
        );

        let (glider_type, comp_id, time_zone) = match (glider_type, comp_id, time_zone) {
            (Some(glider_type), Some(comp_id), Some(time_zone)) =>
                (
                    glider_type,
                    comp_id,
                    match time_zone.parse::<i8>() {
                        Ok(time_zone) => time_zone,
                        Err(_) => return Err(PilotInfoParseError),
                    }
                ),
            _ => return Err(PilotInfoParseError),
        };

        Ok(Self {
            glider_type,
            comp_id,
            time_zone,
        })
    }
}

enum PilotElem {
    GliderType,
    CompetitionId,
    TimeZone,
}

impl PilotElem {
    fn get_element(&self, description: Option<&str>) -> Option<String> {
        let description = description?;
        let start = match self {
            PilotElem::GliderType => "LCU::HPGTYGLIDERTYPE:",
            PilotElem::CompetitionId => "LCU::HPCIDCOMPETITIONID:",
            PilotElem::TimeZone => "LCU::HPTZNTIMEZONE:",
        };

        let regex = Regex::new(format!("{start}.+").as_str()).expect("regex failed to compile");
        let m = regex.find(description.trim())?;

        Some((description.trim()[m.start()+start.len() .. m.end()]).to_string())
    }
}

#[cfg(test)]

mod tests {
    use crate::parser::util;
    use super::*;

    #[test]
    fn pilot_info_ast_parsing() {
        let contents = util::get_contents("examples/ast.igc").expect("failed to read file");
        let pilot_info = PilotInfo::parse(&contents).expect("failed to parse pilot info");
        assert_eq!(pilot_info.comp_id, "KE");
        assert_eq!(pilot_info.time_zone, 2);
        assert_eq!(pilot_info.glider_type, "LS 8");

    }
}