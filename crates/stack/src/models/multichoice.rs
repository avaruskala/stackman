use serde::{Deserialize, Serialize};

use crate::models::common::{Answer, FormattedText, moodle_question};

moodle_question! {
    pub struct MultichoiceQuestion {
        pub single: bool,
        pub shuffleanswers: bool,
        pub answernumbering: AnswerNumbering,
        pub correctfeedback: FormattedText,
        pub partiallycorrectfeedback: FormattedText,
        pub incorrectfeedback: FormattedText,
        pub shownumcorrect: (),
        pub answer: Vec<Answer>,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum AnswerNumbering {
    #[serde(rename = "abc")]
    ABC,
    ABCD,
    #[serde(rename = "123")]
    Number,
    #[serde(rename = "iii")]
    III,
    IIII,
    #[serde(rename = "none")]
    None,
}
