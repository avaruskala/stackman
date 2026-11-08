use serde::{Deserialize, Serialize};

macro_rules! moodle_question {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty,
            )*
        }
    ) => {
        #[::serde_with::serde_as]
        #[::serde_with::skip_serializing_none]
        #[derive(Debug, Clone, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
        #[serde(deny_unknown_fields)]
        $(#[$meta])*
        $vis struct $name {
            pub name: $crate::models::common::Text,
            pub questiontext: $crate::models::common::FormattedText,
            pub generalfeedback: $crate::models::common::FormattedText,
            pub defaultgrade: f32,
            pub penalty: f32,
            #[serde_as(as = "::serde_with::BoolFromInt")]
            pub hidden: bool,
            pub idnumber: Option<String>,
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_ty,
            )*
            pub tags: Option<$crate::models::common::Tags>,
        }
    };
}

pub(crate) use moodle_question;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    #[serde(rename = "@format", default, skip_serializing_if = "Option::is_none")]
    pub format: Option<TextFormat>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TextFormat {
    Html,
    MoodleAutoFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FileEncoding {
    Base64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@path")]
    pub path: String,
    #[serde(rename = "@encoding")]
    pub encoding: FileEncoding,
    #[serde(rename = "$text", default)]
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormattedText {
    #[serde(rename = "@format")]
    pub format: TextFormat,
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file: Vec<File>,
}

/// Puuttuva muotoiltu teksti tulkitaan tyhjäksi HTML-tekstiksi.
impl Default for FormattedText {
    fn default() -> Self {
        Self {
            format: TextFormat::Html,
            text: String::new(),
            file: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tags {
    pub tag: Vec<Text>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    pub text: String,
    #[serde(rename = "@fraction")]
    pub fraction: f32,
    #[serde(rename = "@format")]
    pub format: TextFormat,
    pub feedback: FormattedText,
}
