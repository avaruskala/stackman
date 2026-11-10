use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::{BoolFromInt, serde_as};

use crate::models::common::{FormattedText, Text, moodle_question};

moodle_question! {
    pub struct StackQuestion {
        pub stackversion: Option<Text>,
        pub questionvariables: Text,
        pub specificfeedback: FormattedText,
        pub questionnote: Text,
        pub questiondescription: Option<FormattedText>,
        #[serde_as(as = "BoolFromInt")]
        pub questionsimplify: bool,
        #[serde_as(as = "BoolFromInt")]
        pub assumepositive: bool,
        /// Oletetaan reaaliseksi
        #[serde_as(as = "Option<BoolFromInt>")]
        pub assumereal: Option<bool>,
        #[serde(default)]
        pub prtcorrect: FormattedText,
        #[serde(default)]
        pub prtpartiallycorrect: FormattedText,
        #[serde(default)]
        pub prtincorrect: FormattedText,
        pub decimals: Option<String>,
        pub scientificnotation: Option<String>,
        pub multiplicationsign: MultiplicationSign,
        #[serde_as(as = "BoolFromInt")]
        pub sqrtsign: bool,
        pub complexno: ComplexNo,
        pub inversetrig: InverseTrig,
        pub logicsymbol: Option<String>,
        pub matrixparens: MatrixParens,
        #[serde_as(as = "Option<BoolFromInt>")]
        pub isbroken: Option<bool>,
        /// Määrätyt metatiedot
        pub prescribedmetadata: Option<String>,
        /// Metatiedot JSON-muodossa
        pub metadata: Option<String>,
        /// Kysymysryhmä
        pub variantsselectionseed: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub input: Vec<Input>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub prt: Vec<Prt>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub deployedseed: Vec<u32>,
        /// Kysymystestit
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub qtest: Vec<QTest>,
        pub hint: Option<FormattedText>,
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MultiplicationSign {
    Dot,
    Cross,
    OnSpace,
    None,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ComplexNo {
    I,
    J,
    SymI,
    SymJ,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum InverseTrig {
    #[serde(rename = "cos-1")]
    CosMinus1,
    #[serde(rename = "acos")]
    Acos,
    #[serde(rename = "arccos")]
    Arccos,
    #[serde(rename = "arccos-arcosh")]
    ArccosArcosh,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum MatrixParens {
    #[serde(rename = "[")]
    Square,
    #[serde(rename = "(")]
    Round,
    #[serde(rename = "{")]
    Curly,
    #[serde(rename = "|")]
    Bar,
    #[serde(rename = "")]
    None,
}

/// Tyhjä `<matrixparens></matrixparens>`-elementti tulkitaan arvoksi `None`.
impl<'de> Deserialize<'de> for MatrixParens {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
        match s.trim() {
            "[" => Ok(MatrixParens::Square),
            "(" => Ok(MatrixParens::Round),
            "{" => Ok(MatrixParens::Curly),
            "|" => Ok(MatrixParens::Bar),
            "" => Ok(MatrixParens::None),
            other => Err(serde::de::Error::unknown_variant(
                other,
                &["[", "(", "{", "|", ""],
            )),
        }
    }
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Prt {
    pub name: String,
    /// Kysymyksen arvo
    pub value: f32,
    #[serde_as(as = "BoolFromInt")]
    pub autosimplify: bool,
    pub feedbackstyle: Option<u8>,
    /// Palautteen muuttujat
    pub feedbackvariables: Text,
    pub node: Vec<Node>,
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub name: String,
    pub description: Option<String>,
    /// Vastaustesti
    pub answertest: AnswerTest,
    /// SAns
    pub sans: String,
    /// TAns
    pub tans: String,
    /// Testivaihtoehdot
    pub testoptions: String,
    /// Hiljainen
    #[serde_as(as = "BoolFromInt")]
    pub quiet: bool,

    /// Mod
    pub truescoremode: ScoreMode,
    /// Pisteet
    #[serde(serialize_with = "serialize_score", deserialize_with = "deserialize_score")]
    pub truescore: f32,
    /// Rangaistus
    pub truepenalty: String,
    /// Seuraava
    pub truenextnode: i16,
    /// Vastauksen tunnus
    pub trueanswernote: String,
    /// Palaute, jos vastaus on oikein
    pub truefeedback: FormattedText,

    /// Mod
    pub falsescoremode: ScoreMode,
    /// Pisteet
    #[serde(serialize_with = "serialize_score", deserialize_with = "deserialize_score")]
    pub falsescore: f32,
    /// Rangaistus
    pub falsepenalty: String,
    /// Seuraava
    pub falsenextnode: i16,
    /// Vastauksen tunnus
    pub falseanswernote: String,
    /// Palaute, jos vastaus on oikein
    pub falsefeedback: FormattedText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum ScoreMode {
    /// =
    #[serde(rename = "=")]
    Equals,
    /// +
    #[serde(rename = "+")]
    Plus,
    /// -
    #[serde(rename = "-")]
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AnswerTest {
    AlgEquiv,
    AddConst,
    AlgEquivNouns,
    Antidiff,
    Bool,
    CasEqual,
    CompSquare,
    Diff,
    EqualComAss,
    EqualComAssRules,
    EquivFirst,
    Equiv,
    Expanded,
    FacForm,
    Int,
    Levenshtein,
    LowestTerms,
    GT,
    GTE,
    NumAbsolute,
    NumDecPlaces,
    NumDecPlacesWrong,
    NumRelative,
    NumSigFigs,
    PartFrac,
    PropLogic,
    SameType,
    Sets,
    SetEquiv,
    SigFigsStrict,
    SingleFrac,
    SRegExp,
    String,
    StringSloppy,
    SubstEquiv,
    SysEquiv,
    UnitsAbsolute,
    UnitsRelative,
    Units,
    UnitsStrictAbsolute,
    UnitsStrictRelative,
    UnitsStrict,
    Validator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    Algebraic,
    Equiv,
    GeoGebra,
    JSON,
    Numerical,
    Matrix,
    Varmatrix,
    String,
    Notes,
    Boolean,
    Parsons,
    Dropdown,
    Radio,
    Textarea,
    Checkbox,
    Units,
    Singlechar,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum InsertStars {
    /// Älä lisää tähtiä
    DontInsertStars,
    /// Lisää tähdet ilmeisiin tuloihin.
    InsertStarsForImpliedMultiplicationOnly,
    /// Lisää tähdet olettaen, että muuttujien nimet ovat yhden merkin mittaisia.
    InsertStarsAssumingSingleCharacterVariableNames,
    /// Lisää tähdet ainoastaan välilyöntien kohdalle
    InsertStarsForSpacesOnly,
    /// Lisää tähdet oletettuihin tuloihin ja tyhjiin väleihin.
    InsertStarsForImpliedMultiplicationAndForSpaces,
    /// Lisää tähdet ilmeisiin tuloihin olettaen yksimerkkiset muuttujanimet ja tyhjiin väleihin.
    InsertStarsAssumingSingleCharacterVariablesImpliedAndForSpaces,
    /// Insert stars for implied multiplication, spaces, and no user-functions
    InsertStarsForImpliedMultiplicationSpacesAndNoUserFunctions,
    /// Insert stars for implied multiplication, spaces, no user-functions and assuming single-character var
    InsertStarsForImpliedMultiplicationSpacesNoUserFunctionsAndAssumingSingleCharacterVar,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum SyntaxAttribute {
    Value,
    Placeholder,
}

#[serde_as]
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub name: String,
    /// Vastauksen tyyppi
    #[serde(rename = "type")]
    pub input_type: InputType,
    /// Mallivastaus
    pub tans: String,
    /// Vastauskentän pituus
    pub boxsize: u16,
    #[serde_as(as = "BoolFromInt")]
    pub strictsyntax: bool,
    /// Lisätäänkö tähdet
    pub insertstars: InsertStars,
    /// Syntaksivihje
    pub syntaxhint: String,
    /// Syntaksivihjeen tyyppi
    pub syntaxattribute: Option<SyntaxAttribute>,
    /// Kielletyt merkkijonot
    pub forbidwords: String,
    /// Sallitut sanat
    pub allowwords: String,
    /// Liukuluvut kielletään
    #[serde_as(as = "BoolFromInt")]
    pub forbidfloat: bool,
    /// Vaaditaanko supistettu muoto
    #[serde_as(as = "BoolFromInt")]
    pub requirelowestterms: bool,
    /// Tarkista vastauksen tyyppi
    #[serde_as(as = "BoolFromInt")]
    pub checkanswertype: bool,
    /// Esikatselu
    #[serde_as(as = "BoolFromInt")]
    pub mustverify: bool,
    /// Näytä validointi
    pub showvalidation: ShowValidation,
    /// Lisämääritykset
    pub options: String,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum ShowValidation {
    /// Ei
    No,
    /// Kyllä, muuttujalistan kanssa
    YesWithVariableList,
    /// Kyllä, ilman muuttujalistaa
    YesWithoutVariableList,
    /// Kyllä, tiivis
    YesCompact,
}

fn serialize_score<S: serde::Serializer>(score: &f32, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&score.to_string())
}

/// Pistearvo voi olla desimaaliluku, kokonaisluku tai murtoluku, kuten `1/3` (sekä numerona että merkkijonona).
fn deserialize_score<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    #[derive(Clone, Copy)]
    struct ScoreVisitor;

    impl<'de> serde::de::Visitor<'de> for ScoreVisitor {
        type Value = f32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a float, integer, string, or text element containing a score")
        }

        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
            Ok(v as f32)
        }

        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(v as f32)
        }

        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v as f32)
        }

        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
            let t = s.trim();
            let parsed = match t.split_once('/') {
                Some((num, den)) => num
                    .trim()
                    .parse::<f32>()
                    .ok()
                    .zip(den.trim().parse::<f32>().ok())
                    .filter(|(_, d)| *d != 0.0)
                    .map(|(n, d)| n / d),
                None => t.parse::<f32>().ok(),
            };
            parsed.ok_or_else(|| serde::de::Error::custom(format!("invalid score `{t}`")))
        }

        fn visit_map<M: serde::de::MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut score = None;
            while let Some(key) = map.next_key::<String>()? {
                if key == "$text" {
                    let s: String = map.next_value()?;
                    score = Some(self.visit_str(&s)?);
                } else {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
            score.ok_or_else(|| serde::de::Error::custom("missing score in element"))
        }
    }

    deserializer.deserialize_any(ScoreVisitor)
}

/// Kysymystesti, joka tallennetaan häviöttömästi merkkijonoina.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QTest {
    /// Testitapauksen numero
    pub testcase: String,
    /// Kuvaus
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Testisyötteet
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub testinput: Vec<QTestInput>,
    /// Odotetut tulokset
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected: Vec<QTestExpected>,
}

/// Kysymystestin syöte.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QTestInput {
    pub name: String,
    #[serde(default)]
    pub value: String,
}

/// Kysymystestin odotettu vastauspuun tulos.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QTestExpected {
    pub name: String,
    #[serde(default)]
    pub expectedscore: String,
    #[serde(default)]
    pub expectedpenalty: String,
    #[serde(default)]
    pub expectedanswernote: String,
}
