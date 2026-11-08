use serde::{Deserialize, Serialize};

pub mod common;
pub mod multichoice;
pub mod stack;
pub mod truefalse;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename = "quiz", deny_unknown_fields)]
pub struct Quiz {
    pub question: Vec<Question>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "@type", rename_all = "lowercase")]
pub enum Question {
    Stack(Box<stack::StackQuestion>),
    Multichoice(multichoice::MultichoiceQuestion),
    Category(CategoryQuestion),
    TrueFalse(truefalse::TrueFalseQuestion),
}

impl<'de> Deserialize<'de> for Question {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct QuestionVisitor;

        impl<'de> serde::de::Visitor<'de> for QuestionVisitor {
            type Value = Question;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a <question> element with a type attribute")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut qtype: Option<String> = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "@type" {
                        qtype = Some(map.next_value()?);
                        break;
                    }
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }

                let de = serde::de::value::MapAccessDeserializer::new(map);
                match qtype.as_deref() {
                    Some("stack") => stack::StackQuestion::deserialize(de)
                        .map(Box::new)
                        .map(Question::Stack),
                    Some("category") => CategoryQuestion::deserialize(de).map(Question::Category),
                    Some("multichoice") => {
                        multichoice::MultichoiceQuestion::deserialize(de).map(Question::Multichoice)
                    }
                    Some("truefalse") => {
                        truefalse::TrueFalseQuestion::deserialize(de).map(Question::TrueFalse)
                    }
                    Some(other) => Err(serde::de::Error::unknown_variant(
                        other,
                        &["stack", "category", "multichoice", "truefalse"],
                    )),
                    None => Err(serde::de::Error::missing_field("@type")),
                }
            }
        }

        deserializer.deserialize_map(QuestionVisitor)
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CategoryQuestion {
    pub category: common::Text,
    pub info: Option<common::FormattedText>,
    pub idnumber: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    macro_rules! material {
        ($file:expr $(,)?) => {
            ($file, include_str!(concat!("../../../../material/", $file)))
        };
    }

    fn normalize_json(val: &mut serde_json::Value) {
        match val {
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64()
                    && let Some(num) = serde_json::Number::from_f64(f)
                {
                    *n = num;
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    normalize_json(item);
                }
            }
            serde_json::Value::Object(map) => {
                for v in map.values_mut() {
                    normalize_json(v);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn it_works() {
        // TODO: get test material
        let quiz: [(&str, &str); 0] = [];

        let xml_cfg = quickxml_to_serde::Config::new_with_defaults();

        for (filename, quiz_xml) in quiz {
            let data: Quiz = quick_xml::de::from_str(quiz_xml)
                .unwrap_or_else(|err| panic!("deserialize {filename}: {err:?}"));

            let serialized = quick_xml::se::to_string_with_root("quiz", &data)
                .unwrap_or_else(|err| panic!("serialize {filename}: {err:?}"));

            let roundtrip: Quiz = quick_xml::de::from_str(&serialized)
                .unwrap_or_else(|err| panic!("roundtrip deserialize {filename}: {err:?}"));

            let mut orig_json =
                quickxml_to_serde::xml_string_to_json(quiz_xml.replace("\r\n", "\n"), &xml_cfg)
                    .unwrap_or_else(|err| panic!("orig_json {filename}: {err:?}"));
            let mut ser_json = quickxml_to_serde::xml_string_to_json(serialized.clone(), &xml_cfg)
                .unwrap_or_else(|err| panic!("ser_json {filename}: {err:?}"));

            normalize_json(&mut orig_json);
            normalize_json(&mut ser_json);

            let path = PathBuf::from(format!("test/{filename}"));
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            std::fs::write(&path, &serialized)
                .unwrap_or_else(|err| panic!("write {path}: {err:?}", path = path.display()));

            assert_eq!(orig_json, ser_json, "serialization mismatch for {filename}");
            assert_eq!(data, roundtrip, "roundtrip mismatch for {filename}");
        }
    }
}
