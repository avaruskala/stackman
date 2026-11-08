use crate::models::common::{Answer, moodle_question};

moodle_question! {
    pub struct TrueFalseQuestion {
        pub answer: Vec<Answer>,
    }
}
