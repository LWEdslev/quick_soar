use crate::parser::util::Fix;
use crate::{analysis, parser};

struct Flight {
    segments: Vec<Segment>
}

impl Flight {
    fn make(fixes: Vec<Fix>) -> Self {
        let segments: Vec<Segment> = vec![];
        let start_alt = fixes.get(0).unwrap().alt;
        let fixes = fixes.into_iter().filter(|f| f.alt > start_alt + 100).collect::<Vec<Fix>>();

        const DEGREE_BOUNDARY: f32 = 180.; //turn this many degrees in
        const TIME_WINDOW: i8 = 20;        //this much time


        let mut prev_fix = fixes.get(0).unwrap();
        let mut curr_fix = fixes.get(0).unwrap();
        let mut next_fix = fixes.get(0).unwrap();
        let bearing_changes = fixes.iter().map(|f| {
            prev_fix = curr_fix;
            curr_fix = next_fix;
            next_fix = f;
            analysis::util::bearing_change(prev_fix, curr_fix, next_fix)
        }).collect::<Vec<f32>>();

        for (fix, change) in fixes.iter().zip(bearing_changes) {
            //TODO: hold øje med de sidste 30 sekunder og se om der er en høj nok bearing change ud fra TIME_WINDOW og DEGREE_BOUNDARY
        }

        Self {
            segments: vec![]
        }
    }

    fn thermal_percentage(&self) -> f32 {
        self.segments.iter().filter(
            |s| match s {
                Segment::Glide(_) => false,
                Segment::Thermal(_) => true,
            }
        ).count() as f32 * 100. / self.segments.len() as f32
    }
}

enum Segment {
    Glide(Vec<Fix>),
    Thermal(Vec<Fix>),
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn segmenting_1_sec() {
        let contents = parser::util::get_contents("examples/aat.igc").unwrap();
        let fixes = parser::util::get_fixes(&contents);
        let flight = Flight::make(fixes);
        let target_percentage: f32 = 36.6;
        let acceptance = 4.;
        let range = target_percentage - acceptance .. target_percentage + acceptance;
        assert!(range.contains(&flight.thermal_percentage()));
    }
}

