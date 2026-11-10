use serde::{Deserialize, Serialize};
use stack::models::{CategoryQuestion, Question, Quiz, common::Text};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuestionListView {
    #[default]
    All,
    Chosen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuestionTypeFilter {
    #[default]
    All,
    Stack,
    Multichoice,
    TrueFalse,
}

impl QuestionTypeFilter {
    pub fn matches(self, qtype: &str) -> bool {
        match self {
            Self::All => true,
            Self::Stack => qtype == "stack",
            Self::Multichoice => qtype == "multichoice",
            Self::TrueFalse => qtype == "truefalse",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LanguageFilter {
    #[default]
    All,
    Fi,
    En,
}

impl LanguageFilter {
    pub fn container_class(self) -> &'static str {
        match self {
            Self::All => "preview-content-area lang-filter-all",
            Self::Fi => "preview-content-area lang-filter-fi",
            Self::En => "preview-content-area lang-filter-en",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionEntry {
    pub id: usize,
    pub source_file: String,
    pub category: String,
    pub category_question: CategoryQuestion,
    pub name: String,
    pub qtype: String,
    pub defaultgrade: f32,
    pub penalty: f32,
    pub tags: Vec<String>,
    pub question: Question,
}

impl QuestionEntry {
    pub fn is_stack(&self) -> bool {
        self.qtype == "stack"
    }

    pub fn badge_class(&self) -> &'static str {
        match self.qtype.as_str() {
            "stack" => "qtype-badge badge-stack",
            "multichoice" => "qtype-badge badge-mc",
            "truefalse" => "qtype-badge badge-tf",
            _ => "qtype-badge",
        }
    }

    pub fn badge_short_label(&self) -> &'static str {
        match self.qtype.as_str() {
            "stack" => "STACK",
            "multichoice" => "MC",
            "truefalse" => "T/F",
            _ => "Q",
        }
    }

    pub fn badge_full_label(&self) -> &'static str {
        match self.qtype.as_str() {
            "stack" => "STACK",
            "multichoice" => "Monivalinta",
            "truefalse" => "Tosi tai epätosi",
            _ => "Kysymys",
        }
    }

    pub fn matches_search(&self, lowercase_query: &str) -> bool {
        if lowercase_query.is_empty() {
            return true;
        }
        self.name.to_lowercase().contains(lowercase_query)
            || self.category.to_lowercase().contains(lowercase_query)
            || self.source_file.to_lowercase().contains(lowercase_query)
            || self
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(lowercase_query))
    }
}

pub fn extract_questions_from_quiz(
    quiz: Quiz,
    source_file: &str,
    start_id: usize,
) -> Vec<QuestionEntry> {
    let mut current_category = String::from("Top");
    let mut current_category_question = CategoryQuestion {
        category: Text {
            format: None,
            text: "$course$/top".to_string(),
        },
        info: None,
        idnumber: None,
    };
    let mut entries = Vec::new();
    let mut next_id = start_id;

    for q in quiz.question {
        match q {
            Question::Category(cat) => {
                current_category = clean_category(&cat.category.text);
                current_category_question = cat;
            }
            Question::Stack(sq) => {
                let name = sq.name.text.trim().to_string();
                let defaultgrade = sq.defaultgrade;
                let penalty = sq.penalty;
                let tags = extract_tags(sq.tags.as_ref());
                entries.push(QuestionEntry {
                    id: next_id,
                    source_file: source_file.to_string(),
                    category: current_category.clone(),
                    category_question: current_category_question.clone(),
                    name,
                    qtype: "stack".to_string(),
                    defaultgrade,
                    penalty,
                    tags,
                    question: Question::Stack(sq),
                });
                next_id += 1;
            }
            Question::Multichoice(mq) => {
                let name = mq.name.text.trim().to_string();
                let defaultgrade = mq.defaultgrade;
                let penalty = mq.penalty;
                let tags = extract_tags(mq.tags.as_ref());
                entries.push(QuestionEntry {
                    id: next_id,
                    source_file: source_file.to_string(),
                    category: current_category.clone(),
                    category_question: current_category_question.clone(),
                    name,
                    qtype: "multichoice".to_string(),
                    defaultgrade,
                    penalty,
                    tags,
                    question: Question::Multichoice(mq),
                });
                next_id += 1;
            }
            Question::TrueFalse(tf) => {
                let name = tf.name.text.trim().to_string();
                let defaultgrade = tf.defaultgrade;
                let penalty = tf.penalty;
                let tags = extract_tags(tf.tags.as_ref());
                entries.push(QuestionEntry {
                    id: next_id,
                    source_file: source_file.to_string(),
                    category: current_category.clone(),
                    category_question: current_category_question.clone(),
                    name,
                    qtype: "truefalse".to_string(),
                    defaultgrade,
                    penalty,
                    tags,
                    question: Question::TrueFalse(tf),
                });
                next_id += 1;
            }
        }
    }

    entries
}

pub fn build_export_quiz(entries: &[QuestionEntry]) -> Quiz {
    let mut question = Vec::with_capacity(entries.len() * 2);
    let mut last_category: Option<&CategoryQuestion> = None;

    for entry in entries {
        if last_category != Some(&entry.category_question) {
            question.push(Question::Category(entry.category_question.clone()));
            last_category = Some(&entry.category_question);
        }
        question.push(entry.question.clone());
    }

    Quiz { question }
}

fn extract_tags(tags: Option<&stack::models::common::Tags>) -> Vec<String> {
    tags.map(|t| t.tag.iter().map(|x| x.text.trim().to_string()).collect())
        .unwrap_or_default()
}

pub fn clean_category(raw: &str) -> String {
    let s = raw
        .trim()
        .trim_start_matches("$course$/")
        .trim_start_matches("$module$/")
        .trim_start_matches("top/")
        .trim_start_matches("top");
    if s.is_empty() {
        "Top".to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack::render::{export_quiz_to_moodle_xml, parse_moodle_xml};

    #[test]
    fn test_export_chosen_questions_with_categories() {
        let raw_xml = include_str!(
            "../../../material/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723/questions-Discrete-Multilang.-20161122-0033.xml"
        );
        let quiz = parse_moodle_xml(raw_xml).expect("should parse Discrete Multilang XML");
        let entries = extract_questions_from_quiz(quiz, "questions-Discrete-Multilang.xml", 1);
        assert!(entries.len() >= 3);

        // Select first and last question (potentially from different categories)
        let chosen = vec![entries[0].clone(), entries[entries.len() - 1].clone()];
        let export_quiz = build_export_quiz(&chosen);
        let exported_xml =
            export_quiz_to_moodle_xml(&export_quiz).expect("should serialize export quiz");

        assert!(exported_xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<quiz>"));
        assert!(exported_xml.contains("<question type=\"category\">"));

        let reparsed_quiz =
            parse_moodle_xml(&exported_xml).expect("should reparse exported chosen XML");
        let reparsed_entries = extract_questions_from_quiz(reparsed_quiz, "exported.xml", 1);
        assert_eq!(reparsed_entries.len(), 2);
        assert_eq!(reparsed_entries[0].name, chosen[0].name);
        assert_eq!(reparsed_entries[0].category_question, chosen[0].category_question);
        assert_eq!(reparsed_entries[1].name, chosen[1].name);
        assert_eq!(reparsed_entries[1].category_question, chosen[1].category_question);
        assert_eq!(reparsed_entries[0].question, chosen[0].question);
        assert_eq!(reparsed_entries[1].question, chosen[1].question);
    }
}
