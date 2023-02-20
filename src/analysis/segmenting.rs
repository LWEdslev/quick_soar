use crate::parser::util::Fix;

struct Flight {
    segments: Vec<Segment>
}

impl Flight {
    fn make(fixes: Vec<Fix>) -> Self {
        todo!()
    }
}

enum Segment {
    Glide(Vec<Fix>),
    Thermal(Vec<Fix>),
}

