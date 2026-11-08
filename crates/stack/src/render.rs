use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::models::{
    Question, Quiz,
    common::{FileEncoding, FormattedText, Text, TextFormat},
    multichoice::AnswerNumbering,
    stack::{
        AnswerTest, ComplexNo, Input, InputType, InsertStars, InverseTrig, MatrixParens,
        MultiplicationSign, Node, Prt, ScoreMode, ShowValidation, StackQuestion, SyntaxAttribute,
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedChoice {
    pub value: String,
    pub correct: bool,
    pub display_html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedInput {
    pub name: String,
    pub input_type: InputType,
    pub boxsize: u16,
    pub syntaxhint: String,
    pub syntaxattribute: Option<SyntaxAttribute>,
    pub tans_expr: String,
    pub tans_value: String,
    pub tans_latex: String,
    pub matrix_cells: Option<Vec<Vec<String>>>,
    pub choices: Option<Vec<EvaluatedChoice>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedVariable {
    pub name: String,
    pub value: String,
    pub latex: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedNode {
    pub name: String,
    pub description: Option<String>,
    pub answertest: String,
    pub sans: String,
    pub tans: String,
    pub testoptions: String,
    pub quiet: bool,
    pub truescoremode: String,
    pub truescore: f32,
    pub truenextnode: i16,
    pub trueanswernote: String,
    pub truefeedback_html: String,
    pub falsescoremode: String,
    pub falsescore: f32,
    pub falsenextnode: i16,
    pub falseanswernote: String,
    pub falsefeedback_html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedPrt {
    pub name: String,
    pub value: f32,
    pub autosimplify: bool,
    pub feedbackvariables: String,
    pub nodes: Vec<EvaluatedNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedStackQuestion {
    pub seed: u32,
    pub maxima_ok: bool,
    pub maxima_error: Option<String>,
    pub questiontext_html: String,
    pub generalfeedback_html: String,
    pub specificfeedback_html: String,
    pub questionnote_html: String,
    pub questiondescription_html: Option<String>,
    pub hint_html: Option<String>,
    pub prtcorrect_html: String,
    pub prtpartiallycorrect_html: String,
    pub prtincorrect_html: String,
    pub inputs: Vec<EvaluatedInput>,
    pub variables: Vec<EvaluatedVariable>,
    pub prts: Vec<EvaluatedPrt>,
}

/// Simplified standalone `<question type="stack">` XML format (used in some material/ files)
#[derive(Debug, Deserialize)]
struct SimpleStackQuestionXml {
    #[serde(rename = "@type", default)]
    _qtype: Option<String>,
    name: Text,
    questiontext: FormattedText,
    #[serde(default)]
    questionvariables: Option<String>,
    #[serde(default)]
    teacheranswer: Option<String>,
    #[serde(default)]
    answertests: Option<SimpleAnswerTests>,
}

#[derive(Debug, Deserialize)]
struct SimpleAnswerTests {
    #[serde(default)]
    answertest: Vec<String>,
}

/// Serialize a `Quiz` into Moodle XML format (`<?xml version="1.0" encoding="UTF-8"?>\n<quiz>...`).
pub fn export_quiz_to_moodle_xml(quiz: &Quiz) -> Result<String, String> {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let mut ser = quick_xml::se::Serializer::with_root(&mut xml, Some("quiz"))
        .map_err(|e| format!("XML serializer init error: {e}"))?;
    ser.indent(' ', 2);
    quiz.serialize(ser)
        .map_err(|e| format!("XML serialization error: {e}"))?;
    xml.push('\n');
    Ok(xml)
}

/// Parse either a standard Moodle `<quiz>` XML or a standalone `<question>` XML (including simplified STACK XMLs).
pub fn parse_moodle_xml(xml_str: &str) -> Result<Quiz, String> {
    let trimmed = xml_str.trim();
    if let Ok(quiz) = quick_xml::de::from_str::<Quiz>(trimmed) {
        return Ok(quiz);
    }

    // Try wrapping a single <question> in <quiz>...</quiz>
    if trimmed.starts_with("<question") {
        let wrapped = format!("<quiz>{trimmed}</quiz>");
        if let Ok(quiz) = quick_xml::de::from_str::<Quiz>(&wrapped) {
            return Ok(quiz);
        }

        // Try simplified <question type="stack"> format from material/STACK_*
        match quick_xml::de::from_str::<SimpleStackQuestionXml>(trimmed) {
            Ok(simple) => {
                let mut qvars = simple
                    .questionvariables
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                let raw_tans = simple
                    .teacheranswer
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "ans".to_string());

                // Often teacheranswer is `(ans:diff(...))` or `(ans:(c-b)/a)`
                let tans_var = if raw_tans.starts_with("(ans:") && raw_tans.ends_with(')') {
                    let inner = &raw_tans[1..raw_tans.len() - 1];
                    if !qvars.is_empty() && !qvars.ends_with(';') && !qvars.ends_with('$') {
                        qvars.push(';');
                    }
                    if !qvars.is_empty() {
                        qvars.push('\n');
                    }
                    qvars.push_str(inner);
                    qvars.push(';');
                    "ans".to_string()
                } else {
                    raw_tans
                };

                let mut qtext = simple.questiontext;
                if !qtext.text.contains("[[input:") {
                    qtext
                        .text
                        .push_str("<p>Vastaus: [[input:ans1]] [[validation:ans1]]</p>");
                }

                let answertest = simple
                    .answertests
                    .and_then(|a| a.answertest.into_iter().next())
                    .and_then(|s| {
                        serde_json::from_str::<AnswerTest>(&format!("\"{}\"", s.trim())).ok()
                    })
                    .unwrap_or(AnswerTest::AlgEquiv);

                let empty_html = FormattedText {
                    format: TextFormat::Html,
                    text: String::new(),
                    file: Vec::new(),
                };

                let sq = StackQuestion {
                    name: simple.name,
                    questiontext: qtext,
                    generalfeedback: FormattedText {
                        format: TextFormat::Html,
                        text: format!("<p>Mallivastaus: \\({{@{}@}}\\)</p>", tans_var),
                        file: Vec::new(),
                    },
                    defaultgrade: 1.0,
                    penalty: 0.1,
                    hidden: false,
                    idnumber: None,
                    stackversion: None,
                    questionvariables: Text {
                        format: None,
                        text: qvars,
                    },
                    specificfeedback: FormattedText {
                        format: TextFormat::Html,
                        text: "[[feedback:prt1]]".to_string(),
                        file: Vec::new(),
                    },
                    questionnote: Text {
                        format: None,
                        text: format!("{{@{}@}}", tans_var),
                    },
                    questiondescription: None,
                    questionsimplify: true,
                    assumepositive: false,
                    assumereal: Some(false),
                    prtcorrect: FormattedText {
                        format: TextFormat::Html,
                        text: "<p>Vastaus on oikein!</p>".to_string(),
                        file: Vec::new(),
                    },
                    prtpartiallycorrect: FormattedText {
                        format: TextFormat::Html,
                        text: "<p>Vastaus on osittain oikein.</p>".to_string(),
                        file: Vec::new(),
                    },
                    prtincorrect: FormattedText {
                        format: TextFormat::Html,
                        text: "<p>Vastaus on väärin.</p>".to_string(),
                        file: Vec::new(),
                    },
                    decimals: None,
                    scientificnotation: None,
                    multiplicationsign: MultiplicationSign::Dot,
                    sqrtsign: true,
                    complexno: ComplexNo::I,
                    inversetrig: InverseTrig::CosMinus1,
                    logicsymbol: None,
                    matrixparens: MatrixParens::Square,
                    isbroken: None,
                    variantsselectionseed: String::new(),
                    input: vec![Input {
                        name: "ans1".to_string(),
                        input_type: InputType::Algebraic,
                        tans: tans_var.clone(),
                        boxsize: 15,
                        strictsyntax: true,
                        insertstars: InsertStars::InsertStarsForImpliedMultiplicationOnly,
                        syntaxhint: String::new(),
                        syntaxattribute: Some(SyntaxAttribute::Value),
                        forbidwords: String::new(),
                        allowwords: String::new(),
                        forbidfloat: true,
                        requirelowestterms: false,
                        checkanswertype: false,
                        mustverify: true,
                        showvalidation: ShowValidation::YesWithVariableList,
                        options: String::new(),
                    }],
                    prt: vec![Prt {
                        name: "prt1".to_string(),
                        value: 1.0,
                        autosimplify: true,
                        feedbackstyle: None,
                        feedbackvariables: Text {
                            format: None,
                            text: String::new(),
                        },
                        node: vec![Node {
                            name: "0".to_string(),
                            description: None,
                            answertest,
                            sans: "ans1".to_string(),
                            tans: tans_var,
                            testoptions: String::new(),
                            quiet: false,
                            truescoremode: ScoreMode::Equals,
                            truescore: 1.0,
                            truepenalty: String::new(),
                            truenextnode: -1,
                            trueanswernote: "prt1-1-T".to_string(),
                            truefeedback: empty_html.clone(),
                            falsescoremode: ScoreMode::Equals,
                            falsescore: 0.0,
                            falsepenalty: String::new(),
                            falsenextnode: -1,
                            falseanswernote: "prt1-1-F".to_string(),
                            falsefeedback: empty_html,
                        }],
                    }],
                    deployedseed: Vec::new(),
                    qtest: Vec::new(),
                    prescribedmetadata: None,
                    metadata: None,
                    hint: None,
                    tags: None,
                };

                return Ok(Quiz {
                    question: vec![Question::Stack(Box::new(sq))],
                });
            }
            Err(e) => {
                return Err(format!("Failed to parse question XML: {e}"));
            }
        }
    }

    Err(quick_xml::de::from_str::<Quiz>(trimmed)
        .unwrap_err()
        .to_string())
}

/// Replace `@@PLUGINFILE@@/...` URLs with embedded base64 data URLs.
pub fn resolve_pluginfiles(ft: &FormattedText) -> String {
    let mut out = ft.text.clone();
    for file in &ft.file {
        let FileEncoding::Base64 = file.encoding;
        let mime = guess_mime(&file.name);
        let b64 = file.data.split_whitespace().collect::<String>();
        let data_url = format!("data:{mime};base64,{b64}");

        let path_trimmed = file.path.trim_matches('/');
        if !path_trimmed.is_empty() {
            let pat_with_path = format!("@@PLUGINFILE@@/{}/{}", path_trimmed, file.name);
            out = out.replace(&pat_with_path, &data_url);
        }
        let pat_direct = format!("@@PLUGINFILE@@/{}", file.name);
        out = out.replace(&pat_direct, &data_url);
    }
    out
}

fn guess_mime(filename: &str) -> &'static str {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "application/octet-stream"
    }
}

/// Clean Maxima's default `\ifx\endpmatrix...` output into standard AMS-LaTeX matrix environments
/// that MathJax renders natively, respecting the question's `matrixparens` setting.
pub fn clean_maxima_tex(tex: &str, matrixparens: MatrixParens) -> String {
    let (begin_env, end_env) = match matrixparens {
        MatrixParens::Square => ("\\begin{bmatrix}", "\\end{bmatrix}"),
        MatrixParens::Round => ("\\begin{pmatrix}", "\\end{pmatrix}"),
        MatrixParens::Curly => ("\\begin{Bmatrix}", "\\end{Bmatrix}"),
        MatrixParens::Bar => ("\\begin{vmatrix}", "\\end{vmatrix}"),
        MatrixParens::None => ("\\begin{matrix}", "\\end{matrix}"),
    };

    tex.replace(
        "\\ifx\\endpmatrix\\undefined\\pmatrix{\\else\\begin{pmatrix}\\fi ",
        begin_env,
    )
    .replace(
        "\\ifx\\endpmatrix\\undefined\\pmatrix{\\else\\begin{pmatrix}\\fi",
        begin_env,
    )
    .replace(
        "\\ifx\\endpmatrix\\undefined}\\else\\end{pmatrix}\\fi",
        end_env,
    )
    .replace("\\cr ", "\\\\ ")
    .replace("\\cr", "\\\\")
    .trim()
    .to_string()
}

/// Normalize STACK `questionvariables` code (which may use newlines or semicolons as statement separators)
/// into valid `$` terminated Maxima statements.
pub fn normalize_maxima_statements(code: &str) -> Vec<String> {
    let stripped = strip_maxima_comments(code);
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut in_str = false;
    let mut escape = false;
    let mut cur = String::new();

    for ch in stripped.chars() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            cur.push(ch);
            escape = true;
            continue;
        }
        if ch == '"' {
            in_str = !in_str;
            cur.push(ch);
            continue;
        }
        if !in_str {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = (depth - 1).max(0),
                ';' | '$' if depth == 0 => {
                    let stmt = cur.trim();
                    if !stmt.is_empty() {
                        out.push(stmt.to_string());
                    }
                    cur.clear();
                    continue;
                }
                '\n' if depth == 0 => {
                    let stmt = cur.trim();
                    if !stmt.is_empty() && !ends_with_continuation(stmt) {
                        out.push(stmt.to_string());
                        cur.clear();
                    } else if !cur.is_empty() {
                        cur.push(' ');
                    }
                    continue;
                }
                _ => {}
            }
        }
        cur.push(ch);
    }

    let tail = cur.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

fn strip_maxima_comments(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;
    let mut in_str = false;
    let mut comment_depth: usize = 0;

    while i < chars.len() {
        if comment_depth > 0 {
            if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                comment_depth -= 1;
                i += 2;
                continue;
            }
            if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
                comment_depth += 1;
                i += 2;
                continue;
            }
            if chars[i] == '\n' {
                out.push('\n');
            }
            i += 1;
            continue;
        }

        if chars[i] == '\\' && i + 1 < chars.len() {
            out.push(chars[i]);
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }

        if chars[i] == '"' {
            in_str = !in_str;
            out.push(chars[i]);
            i += 1;
            continue;
        }

        if !in_str && i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            comment_depth += 1;
            i += 2;
            continue;
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

fn ends_with_continuation(stmt: &str) -> bool {
    let s = stmt.trim_end();
    for op in [":=", ":", "+", "-", "*", "/", ",", "=", "<", ">"] {
        if s.ends_with(op) {
            return true;
        }
    }
    for kw in ["then", "else", "do", "and", "or", "not"] {
        if let Some(prefix) = s.strip_suffix(kw)
            && (prefix.is_empty() || !prefix.chars().last().unwrap().is_ascii_alphanumeric())
        {
            return true;
        }
    }
    false
}

/// Extract top-level assigned variable names from normalized `questionvariables` statements.
pub fn extract_assigned_variables(statements: &[String]) -> Vec<String> {
    let mut vars = Vec::new();
    let mut seen = BTreeSet::new();
    for stmt in statements {
        // Look for `ident : expr` (but not `ident := expr` or `ident[i] : expr`)
        if let Some(colon_idx) = stmt.find(':') {
            let after = &stmt[colon_idx + 1..];
            if after.starts_with('=') {
                continue;
            }
            let lhs = stmt[..colon_idx].trim();
            if !lhs.is_empty()
                && lhs.chars().next().unwrap().is_ascii_alphabetic()
                && lhs.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && seen.insert(lhs.to_string())
            {
                vars.push(lhs.to_string());
            }
        }
    }
    vars
}

/// Remove all `[[comment]]...[[/comment]]` blocks (including nested or sequential ones).
fn strip_castext_comments(text: &str) -> String {
    let mut current = text.to_string();
    loop {
        let mut search_from = 0;
        let mut replaced = false;
        while let Some(rel_end) = current[search_from..].find("[[/comment]]") {
            let end_pos = search_from + rel_end;
            if let Some(start_pos) = current[..end_pos].rfind("[[comment]]") {
                current.replace_range(start_pos..end_pos + 12, "");
                replaced = true;
                break;
            } else {
                search_from = end_pos + 12;
            }
        }
        if !replaced {
            break;
        }
    }
    current
}

#[derive(Debug)]
struct CastextBlockMatch {
    start: usize,
    end: usize,
    attrs: String,
    body: String,
    self_closing: bool,
}

fn find_opening_tag_end(text: &str, after_tag_idx: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = after_tag_idx;
    let mut in_quote: Option<u8> = None;
    while i + 1 < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_quote {
            if b == q {
                in_quote = None;
            }
        } else if b == b'"' || b == b'\'' {
            in_quote = Some(b);
        } else if b == b']' && bytes[i + 1] == b']' {
            return Some(i);
        }
        i += 1;
    }
    text[after_tag_idx..].find("]]").map(|rel| after_tag_idx + rel)
}

fn find_castext_block(text: &str, tag_name: &str, mut search_from: usize) -> Option<CastextBlockMatch> {
    let open_pat = format!("[[{tag_name}");
    let close_pat = format!("[[/{tag_name}]]");
    let bytes = text.as_bytes();

    while search_from < bytes.len() {
        let rel_idx = text[search_from..].find(&open_pat)?;
        let start = search_from + rel_idx;
        let after_tag_idx = start + open_pat.len();
        if after_tag_idx >= bytes.len() {
            return None;
        }
        let next_b = bytes[after_tag_idx];
        if next_b != b']' && next_b != b'/' && !next_b.is_ascii_whitespace() {
            search_from = after_tag_idx;
            continue;
        }

        let Some(header_end) = find_opening_tag_end(text, after_tag_idx) else {
            return None;
        };
        let raw_attrs = &text[after_tag_idx..header_end];
        let trimmed_attrs = raw_attrs.trim_end();
        if let Some(stripped_attrs) = trimmed_attrs.strip_suffix('/') {
            return Some(CastextBlockMatch {
                start,
                end: header_end + 2,
                attrs: stripped_attrs.to_string(),
                body: String::new(),
                self_closing: true,
            });
        }

        let body_start = header_end + 2;
        if let Some(rel_close) = text[body_start..].find(&close_pat) {
            let body_end = body_start + rel_close;
            let end = body_end + close_pat.len();
            return Some(CastextBlockMatch {
                start,
                end,
                attrs: raw_attrs.to_string(),
                body: text[body_start..body_end].to_string(),
                self_closing: false,
            });
        } else {
            search_from = body_start;
        }
    }
    None
}

/// Replace `[[escape]]...[[/escape]]` and `[[escape value="..."/]]` blocks with placeholders.
fn extract_castext_escapes(text: &str) -> (String, Vec<String>) {
    let mut current = text.to_string();
    let mut escapes = Vec::new();
    let mut search_from = 0;

    while let Some(m) = find_castext_block(&current, "escape", search_from) {
        let val = if let Some(v) = extract_attr(&m.attrs, "value") {
            v
        } else {
            m.body
        };
        let placeholder = format!("\x00STACK_ESCAPE_{}\x00", escapes.len());
        escapes.push(val);
        current.replace_range(m.start..m.end, &placeholder);
        search_from = m.start + placeholder.len();
    }

    (current, escapes)
}

fn restore_castext_escapes(mut text: String, escapes: &[String]) -> String {
    for (idx, val) in escapes.iter().enumerate() {
        let placeholder = format!("\x00STACK_ESCAPE_{idx}\x00");
        text = text.replace(&placeholder, val);
    }
    text
}

/// Expand `[[template ...]]...[[/template]]` and `[[template ... /]]` blocks.
fn process_template_blocks(text: &str) -> String {
    let mut current = text.to_string();
    let mut templates: HashMap<String, String> = HashMap::new();

    // Pass 1: collect non-default template definitions with non-empty content and remove them
    let mut search_from = 0;
    while let Some(m) = find_castext_block(&current, "template", search_from) {
        let mode = extract_attr(&m.attrs, "mode");
        let name = extract_attr(&m.attrs, "name");
        if !m.self_closing && !m.body.is_empty() && mode.as_deref() != Some("default") {
            if let Some(n) = name {
                templates.insert(n, m.body);
            }
            current.replace_range(m.start..m.end, "");
            search_from = m.start;
        } else {
            search_from = m.end;
        }
    }

    // Pass 2: expand remaining template usages and mode="default" fallbacks
    for _ in 0..8 {
        if !current.contains("[[template") {
            break;
        }
        let mut changed = false;
        let mut pos = 0;
        while let Some(m) = find_castext_block(&current, "template", pos) {
            let mode = extract_attr(&m.attrs, "mode");
            let name = extract_attr(&m.attrs, "name");
            let replacement = if let Some(ref n) = name
                && let Some(val) = templates.get(n)
            {
                val.clone()
            } else if mode.as_deref() == Some("default") {
                m.body
            } else {
                String::new()
            };
            current.replace_range(m.start..m.end, &replacement);
            pos = m.start + replacement.len();
            changed = true;
        }
        if !changed {
            break;
        }
    }

    current
}

/// Extract all `{@expr@}`, `@{expr}`, `{#expr#}`, and `[[if test="expr"]]` expressions from a text.
pub fn collect_castext_expressions(
    text: &str,
    tex_exprs: &mut BTreeSet<String>,
    str_exprs: &mut BTreeSet<String>,
    if_exprs: &mut BTreeSet<String>,
) {
    let stripped = strip_castext_comments(text);
    let (escaped, _escapes) = extract_castext_escapes(&stripped);
    let expanded = process_template_blocks(&escaped);
    let text = expanded.as_str();

    // 1. {@ ... @}
    let mut rest = text;
    while let Some(start) = rest.find("{@") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find("@}") {
            let expr = after[..end].trim();
            if !expr.is_empty() {
                tex_exprs.insert(expr.to_string());
            }
            rest = &after[end + 2..];
        } else {
            break;
        }
    }

    // 2. @{ ... }
    let mut rest = text;
    while let Some(start) = rest.find("@{") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find('}') {
            let expr = after[..end].trim();
            if !expr.is_empty() {
                tex_exprs.insert(expr.to_string());
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }

    // 3. {# ... #}
    let mut rest = text;
    while let Some(start) = rest.find("{#") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find("#}") {
            let expr = after[..end].trim();
            if !expr.is_empty() {
                str_exprs.insert(expr.to_string());
            }
            rest = &after[end + 2..];
        } else {
            break;
        }
    }

    // 4. [[if test="..."]] and [[elif test="..."]]
    for tag_prefix in ["[[if ", "[[elif "] {
        let mut rest = text;
        while let Some(start) = rest.find(tag_prefix) {
            let after = &rest[start + tag_prefix.len()..];
            if let Some(end) = after.find("]]") {
                let attr_str = &after[..end];
                if let Some(test_expr) = extract_attr(attr_str, "test") {
                    if_exprs.insert(test_expr);
                }
                rest = &after[end + 2..];
            } else {
                break;
            }
        }
    }
}

fn extract_attr(attrs: &str, attr_name: &str) -> Option<String> {
    let bytes = attrs.as_bytes();
    let name_bytes = attr_name.as_bytes();
    let mut i = 0;
    while i + name_bytes.len() <= bytes.len() {
        if &bytes[i..i + name_bytes.len()] == name_bytes {
            let valid_start = i == 0 || bytes[i - 1].is_ascii_whitespace();
            if valid_start {
                let mut j = i + name_bytes.len();
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'=' {
                    j += 1;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                        let quote = bytes[j] as char;
                        let val_start = j + 1;
                        if let Some(end_offset) = attrs[val_start..].find(quote) {
                            return Some(attrs[val_start..val_start + end_offset].to_string());
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

fn extract_input_refs(attrs: &str) -> Vec<(String, String)> {
    let mut refs = Vec::new();
    let bytes = attrs.as_bytes();
    let prefix = b"input-ref-";
    let mut i = 0;
    while i + prefix.len() < bytes.len() {
        if &bytes[i..i + prefix.len()] == prefix
            && (i == 0 || bytes[i - 1].is_ascii_whitespace())
        {
            let name_start = i + prefix.len();
            let mut j = name_start;
            while j < bytes.len() && !bytes[j].is_ascii_whitespace() && bytes[j] != b'=' {
                j += 1;
            }
            let inp_name = &attrs[name_start..j];
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if !inp_name.is_empty() && j < bytes.len() && bytes[j] == b'=' {
                j += 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                    let quote = bytes[j] as char;
                    let val_start = j + 1;
                    if let Some(end_offset) = attrs[val_start..].find(quote) {
                        let var_name = &attrs[val_start..val_start + end_offset];
                        refs.push((inp_name.to_string(), var_name.to_string()));
                        i = val_start + end_offset + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    refs
}

fn replace_cors_url(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("cors://") {
        format!("/corsscripts/{rest}")
    } else {
        url.to_string()
    }
}

fn replace_cors_in_js(code: &str) -> String {
    code.replace("'cors://", "'/corsscripts/")
        .replace("\"cors://", "\"/corsscripts/")
}

/// Separate ES module static `import` statements from the rest of the JavaScript code,
/// corresponding to `stack_castext2_javascript::separate_imports`.
pub fn separate_js_imports(code: &str) -> (String, String) {
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut import_ranges: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    let mut at_stmt_start = true;

    while i < len {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            if b == b'\n' || b == b'\r' {
                at_stmt_start = true;
            }
            i += 1;
            continue;
        }
        // Line comment
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            i += 2;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            at_stmt_start = true;
            continue;
        }
        // Block comment
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(len);
            continue;
        }
        // String literal outside import
        if b == b'\'' || b == b'"' || b == b'`' {
            let quote = b;
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            at_stmt_start = false;
            continue;
        }

        if at_stmt_start && code[i..].starts_with("import") {
            let after_kw = i + 6;
            if after_kw < len {
                let next_b = bytes[after_kw];
                if next_b.is_ascii_whitespace()
                    || next_b == b'*'
                    || next_b == b'{'
                    || next_b == b'\''
                    || next_b == b'"'
                {
                    let mut k = after_kw;
                    while k < len && bytes[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    // Exclude dynamic `import(...)` and `import.meta`
                    if k < len && bytes[k] != b'(' && bytes[k] != b'.' {
                        let stmt_start = i;
                        let mut j = k;
                        let mut brace_depth: i32 = 0;
                        let mut stmt_end = len;
                        while j < len {
                            let cb = bytes[j];
                            if cb == b'{' {
                                brace_depth += 1;
                                j += 1;
                            } else if cb == b'}' {
                                brace_depth = (brace_depth - 1).max(0);
                                j += 1;
                            } else if cb == b'\'' || cb == b'"' {
                                let q = cb;
                                j += 1;
                                while j < len {
                                    if bytes[j] == b'\\' {
                                        j += 2;
                                        continue;
                                    }
                                    if bytes[j] == q {
                                        j += 1;
                                        break;
                                    }
                                    j += 1;
                                }
                                if brace_depth == 0 {
                                    while j < len && (bytes[j] == b' ' || bytes[j] == b'\t') {
                                        j += 1;
                                    }
                                    if j < len && bytes[j] == b';' {
                                        j += 1;
                                    }
                                    stmt_end = j;
                                    break;
                                }
                            } else if cb == b';' && brace_depth == 0 {
                                stmt_end = j + 1;
                                break;
                            } else {
                                j += 1;
                            }
                        }
                        import_ranges.push((stmt_start, stmt_end));
                        i = stmt_end;
                        at_stmt_start = true;
                        continue;
                    }
                }
            }
        }

        if b == b';' {
            at_stmt_start = true;
        } else {
            at_stmt_start = false;
        }
        i += 1;
    }

    if import_ranges.is_empty() {
        return (String::new(), replace_cors_in_js(code));
    }

    let mut imports_vec = Vec::with_capacity(import_ranges.len());
    let mut rem = String::with_capacity(code.len());
    let mut last_idx = 0;
    for (s, e) in import_ranges {
        rem.push_str(&code[last_idx..s]);
        let imp = replace_cors_in_js(code[s..e].trim());
        imports_vec.push(imp);
        last_idx = e;
    }
    rem.push_str(&code[last_idx..]);

    (
        imports_vec.join("\n"),
        replace_cors_in_js(rem.trim_start()),
    )
}

/// Extract inner `[[style ...]]` and `[[script ...]]` blocks from an `[[iframe]]`, `[[javascript]]`, or `[[jsxgraph]]` body.
fn extract_styles_and_scripts(body: &str) -> (String, String, String) {
    let mut current = body.to_string();
    let mut styles_html = String::new();
    let mut scripts_html = String::new();

    // 1. Extract [[style ...]]
    let mut search_from = 0;
    while let Some(m) = find_castext_block(&current, "style", search_from) {
        let mut extra_attrs = String::new();
        for attr_name in ["media", "blocking", "title", "nonce", "type", "crossorigin"] {
            if let Some(val) = extract_attr(&m.attrs, attr_name) {
                extra_attrs.push_str(&format!(" {attr_name}=\"{}\"", escape_html_attr(&val)));
            }
        }
        if let Some(href_raw) = extract_attr(&m.attrs, "href") {
            let href = replace_cors_url(&href_raw);
            styles_html.push_str(&format!(
                "<link rel=\"stylesheet\" href=\"{}\"{extra_attrs}/>",
                escape_html_attr(&href)
            ));
            if !m.body.trim().is_empty() {
                styles_html.push_str(&format!("<style{extra_attrs}>{}</style>", m.body));
            }
        } else if !m.body.trim().is_empty() {
            styles_html.push_str(&format!("<style{extra_attrs}>{}</style>", m.body));
        }
        current.replace_range(m.start..m.end, "");
        search_from = m.start;
    }

    // 2. Extract [[script ...]]
    let mut search_from = 0;
    while let Some(m) = find_castext_block(&current, "script", search_from) {
        let mut script_attrs = String::new();
        for attr_name in ["type", "blocking", "src", "integrity", "nonce", "async"] {
            if let Some(mut val) = extract_attr(&m.attrs, attr_name) {
                if attr_name == "src" {
                    val = replace_cors_url(&val);
                }
                script_attrs.push_str(&format!(" {attr_name}=\"{}\"", escape_html_attr(&val)));
            }
        }
        scripts_html.push_str(&format!("<script{script_attrs}>{}</script>", m.body));
        current.replace_range(m.start..m.end, "");
        search_from = m.start;
    }

    (styles_html, scripts_html, current)
}

fn compute_frame_style(attrs: &str, allow_hidden: bool) -> String {
    let has_height = extract_attr(attrs, "height").is_some();
    let width = extract_attr(attrs, "width").unwrap_or_else(|| "500px".to_string());
    let height = extract_attr(attrs, "height").unwrap_or_else(|| "400px".to_string());

    let mut style = if let Some(ar) = extract_attr(attrs, "aspect-ratio") {
        if has_height {
            format!("height:{height};aspect-ratio:{ar};")
        } else {
            format!("width:{width};aspect-ratio:{ar};")
        }
    } else {
        format!("width:{width};height:{height};")
    };

    if allow_hidden
        && let Some(hidden) = extract_attr(attrs, "hidden")
        && (hidden == "true" || hidden == "1")
    {
        style.push_str("display:none;");
    }

    if let Some(extra_style) = extract_attr(attrs, "style") {
        style.push_str(&format!("{extra_style};"));
    }

    style
}

fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Evaluate `[[if test="..."]] ... [[elif test="..."]] ... [[else]] ... [[/if]]` blocks in CASText.
fn process_if_blocks(text: &str, if_map: &BTreeMap<String, bool>) -> String {
    let mut current = text.to_string();
    // Process innermost [[if ...]]...[[/if]] blocks iteratively
    while let Some(end_pos) = current.find("[[/if]]") {
        let Some(start_pos) = current[..end_pos].rfind("[[if ") else {
            break;
        };
        let block = &current[start_pos..end_pos + 7];
        let replacement = eval_single_if_block(block, if_map);
        current.replace_range(start_pos..end_pos + 7, &replacement);
    }
    current
}

fn eval_single_if_block(block: &str, if_map: &BTreeMap<String, bool>) -> String {
    // Strip outer [[if ...]] and [[/if]]
    let Some(header_end) = block.find("]]") else {
        return String::new();
    };
    let header_attrs = &block[5..header_end];
    let first_cond = extract_attr(header_attrs, "test")
        .and_then(|t| if_map.get(&t).copied())
        .unwrap_or(false);

    let inner = &block[header_end + 2..block.len() - 7];

    // Check if there are [[elif ...]] or [[else]] inside `inner`
    if !inner.contains("[[elif ") && !inner.contains("[[else]]") {
        return if first_cond {
            inner.to_string()
        } else {
            String::new()
        };
    }

    // Parse branches
    struct Branch {
        cond: Option<bool>,
        body: String,
    }
    let mut branches = Vec::new();
    let mut cur_cond = Some(first_cond);
    let mut rest = inner;

    loop {
        let next_elif = rest.find("[[elif ");
        let next_else = rest.find("[[else]]");
        match (next_elif, next_else) {
            (Some(ei), None) => {
                let body = &rest[..ei];
                branches.push(Branch {
                    cond: cur_cond,
                    body: body.to_string(),
                });
                let after_elif = &rest[ei + 7..];
                if let Some(tag_end) = after_elif.find("]]") {
                    let attrs = &after_elif[..tag_end];
                    cur_cond = Some(
                        extract_attr(attrs, "test")
                            .and_then(|t| if_map.get(&t).copied())
                            .unwrap_or(false),
                    );
                    rest = &after_elif[tag_end + 2..];
                } else {
                    break;
                }
            }
            (Some(ei), Some(el)) if ei < el => {
                let body = &rest[..ei];
                branches.push(Branch {
                    cond: cur_cond,
                    body: body.to_string(),
                });
                let after_elif = &rest[ei + 7..];
                if let Some(tag_end) = after_elif.find("]]") {
                    let attrs = &after_elif[..tag_end];
                    cur_cond = Some(
                        extract_attr(attrs, "test")
                            .and_then(|t| if_map.get(&t).copied())
                            .unwrap_or(false),
                    );
                    rest = &after_elif[tag_end + 2..];
                } else {
                    break;
                }
            }
            (_, Some(el)) => {
                let body = &rest[..el];
                branches.push(Branch {
                    cond: cur_cond,
                    body: body.to_string(),
                });
                cur_cond = None;
                rest = &rest[el + 8..];
            }
            (None, None) => {
                branches.push(Branch {
                    cond: cur_cond,
                    body: rest.to_string(),
                });
                break;
            }
        }
    }

    for b in branches {
        if b.cond.unwrap_or(true) {
            return b.body;
        }
    }
    String::new()
}

/// Check if byte offset `pos` in `text` is already inside a LaTeX math environment (`\(...\)`, `\[...\]`, `$$...$$`, `\begin{align*}...`).
fn is_inside_math_mode(prefix: &str) -> bool {
    let last_inline_open = prefix.rfind("\\(");
    let last_inline_close = prefix.rfind("\\)");
    if let Some(o) = last_inline_open
        && last_inline_close.is_none_or(|c| o > c)
    {
        return true;
    }

    let last_disp_open = prefix.rfind("\\[");
    let last_disp_close = prefix.rfind("\\]");
    if let Some(o) = last_disp_open
        && last_disp_close.is_none_or(|c| o > c)
    {
        return true;
    }

    let last_env_open = prefix.rfind("\\begin{");
    let last_env_close = prefix.rfind("\\end{");
    if let Some(o) = last_env_open
        && last_env_close.is_none_or(|c| o > c)
    {
        return true;
    }

    false
}

/// Render a CASText string with evaluated Maxima expressions, JSXGraph blocks, input widgets, and feedback tags.
pub fn render_castext(
    ft: &FormattedText,
    tex_map: &BTreeMap<String, (bool, String)>,
    str_map: &BTreeMap<String, String>,
    if_map: &BTreeMap<String, bool>,
    inputs: &[EvaluatedInput],
    matrixparens: MatrixParens,
    jxg_counter: &mut usize,
) -> String {
    let mut text = resolve_pluginfiles(ft);

    // 0a. Strip [[comment]]...[[/comment]]
    text = strip_castext_comments(&text);

    // 0b. Extract [[escape]]...[[/escape]] and [[escape value="..."/]] into placeholders
    let (escaped_text, escapes) = extract_castext_escapes(&text);
    text = escaped_text;

    // 0c. Expand [[template ...]] blocks
    text = process_template_blocks(&text);

    // 1. Process [[if ...]] blocks
    text = process_if_blocks(&text, if_map);

    // 2. Replace {# expr #}
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(start) = rest.find("{#") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("#}") {
            let expr = after[..end].trim();
            if let Some(val) = str_map.get(expr) {
                out.push_str(val);
            } else {
                out.push_str(expr);
            }
            rest = &after[end + 2..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        }
    }
    out.push_str(rest);
    text = out;

    // 3. Replace {@ expr @}
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(start) = rest.find("{@") {
        out.push_str(&rest[..start]);
        let in_math = is_inside_math_mode(&out);
        let after = &rest[start + 2..];
        if let Some(end) = after.find("@}") {
            let expr = after[..end].trim();
            if let Some((is_str, tex_val)) = tex_map.get(expr) {
                if *is_str {
                    out.push_str(tex_val);
                } else {
                    let cleaned = clean_maxima_tex(tex_val, matrixparens);
                    if in_math {
                        out.push_str(&cleaned);
                    } else {
                        out.push_str("\\(");
                        out.push_str(&cleaned);
                        out.push_str("\\)");
                    }
                }
            } else if in_math {
                out.push_str(&format!("\\mathtt{{{}}}", expr.replace('_', "\\_")));
            } else {
                out.push_str(&format!("\\(\\mathtt{{{}}}\\)", expr.replace('_', "\\_")));
            }
            rest = &after[end + 2..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        }
    }
    out.push_str(rest);
    text = out;

    // 4. Replace @{ expr } (used in simplified STACK XMLs)
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(start) = rest.find("@{") {
        out.push_str(&rest[..start]);
        let in_math = is_inside_math_mode(&out);
        let after = &rest[start + 2..];
        if let Some(end) = after.find('}') {
            let expr = after[..end].trim();
            if let Some((is_str, tex_val)) = tex_map.get(expr) {
                if *is_str {
                    out.push_str(tex_val);
                } else {
                    let cleaned = clean_maxima_tex(tex_val, matrixparens);
                    if in_math {
                        out.push_str(&cleaned);
                    } else {
                        out.push_str("\\(");
                        out.push_str(&cleaned);
                        out.push_str("\\)");
                    }
                }
            } else {
                out.push_str(expr);
            }
            rest = &after[end + 1..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        }
    }
    out.push_str(rest);
    text = out;

    // 4a. Process [[quid ...]]
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "quid", pos) {
        let replacement = extract_attr(&m.attrs, "id")
            .map(|id| format!("stack-quid-{id}"))
            .unwrap_or_default();
        text.replace_range(m.start..m.end, &replacement);
        pos = m.start + replacement.len();
    }

    // 4b. Process [[cors ...]]
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "cors", pos) {
        let replacement = extract_attr(&m.attrs, "src")
            .map(|src| format!("/corsscripts/{src}"))
            .unwrap_or_default();
        text.replace_range(m.start..m.end, &replacement);
        pos = m.start + replacement.len();
    }

    // 4c. Process [[jsstring]]...[[/jsstring]]
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "jsstring", pos) {
        let replacement = serde_json::to_string(&m.body).unwrap_or_else(|_| "\"\"".to_string());
        text.replace_range(m.start..m.end, &replacement);
        pos = m.start + replacement.len();
    }

    // 4d. Process [[entityescape]]...[[/entityescape]]
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "entityescape", pos) {
        let replacement = escape_html_attr(&m.body);
        text.replace_range(m.start..m.end, &replacement);
        pos = m.start + replacement.len();
    }

    // 5a. Process [[iframe ...]]...[[/iframe]]
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "iframe", pos) {
        *jxg_counter += 1;
        let idx = *jxg_counter;
        let frame_id = format!("stack-iframe-{idx}");
        let holder_id = format!("stack-iframe-holder-{idx}");
        let title = extract_attr(&m.attrs, "title")
            .unwrap_or_else(|| format!("STACK IFRAME {idx}"));
        let scrolling = match extract_attr(&m.attrs, "scrolling").as_deref() {
            Some("false") | Some("0") => "false",
            _ => "true",
        };
        let astyle = compute_frame_style(&m.attrs, true);
        let (styles, scripts, clean_body) = extract_styles_and_scripts(&m.body);
        let body_or_nbsp = if clean_body.trim().is_empty() {
            "&#160;"
        } else {
            clean_body.as_str()
        };
        let escaped_title = escape_html_attr(&title);
        let scrolling_attr = if scrolling == "false" { "no" } else { "yes" };
        let srcdoc = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Strict//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\">\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head><title>{escaped_title}</title>{styles}<script>const FRAME_ID = \"{frame_id}\";</script>{scripts}</head>\n<body style=\"margin:0px;\">{body_or_nbsp}</body></html>"
        );
        let replacement = format!(
            "<div id=\"{holder_id}\" class=\"stack-iframe-holder\" style=\"{astyle}\" data-frame-id=\"{frame_id}\" data-frame-title=\"{escaped_title}\" data-frame-scrolling=\"{scrolling}\" data-frame-srcdoc=\"{srcdoc}\"><iframe id=\"{frame_id}\" class=\"stack-iframe\" style=\"width:100%;height:100%;border:0;\" scrolling=\"{scrolling_attr}\" title=\"{escaped_title}\" referrerpolicy=\"no-referrer\" sandbox=\"allow-scripts allow-downloads\" srcdoc=\"{srcdoc}\"></iframe></div>",
            astyle = escape_html_attr(&astyle),
            srcdoc = escape_html_attr(&srcdoc),
        );
        text.replace_range(m.start..m.end, &replacement);
        pos = m.start + replacement.len();
    }

    // 5b. Process [[javascript ...]]...[[/javascript]]
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "javascript", pos) {
        *jxg_counter += 1;
        let idx = *jxg_counter;
        let frame_id = format!("stack-iframe-{idx}");
        let holder_id = format!("stack-iframe-holder-{idx}");
        let title = format!("STACK javascript {idx}");
        let escaped_title = escape_html_attr(&title);
        let (styles, mut scripts, clean_body) = extract_styles_and_scripts(&m.body);
        let refs = extract_input_refs(&m.attrs);

        if !refs.is_empty() {
            let (imports, rest_body) = separate_js_imports(&clean_body);
            let promises = refs
                .iter()
                .map(|(inp, _)| format!("stack_js.request_access_to_input(\"{}\",true)", inp))
                .collect::<Vec<_>>()
                .join(",");
            let vars = refs
                .iter()
                .map(|(_, var)| var.as_str())
                .collect::<Vec<_>>()
                .join(",");
            scripts.push_str(&format!(
                "<script type=\"module\">\nimport stack_js from '/corsscripts/stackjsiframe.min.js';\n{imports}\nPromise.all([{promises}]).then(([{vars}]) => {{\n{rest_body}\n}});\n</script>"
            ));
        } else {
            let body_js = replace_cors_in_js(&clean_body);
            scripts.push_str(&format!(
                "<script type=\"module\">\nimport stack_js from '/corsscripts/stackjsiframe.min.js';\n{body_js}\n</script>"
            ));
        }

        let srcdoc = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Strict//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\">\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head><title>{escaped_title}</title>{styles}<script>const FRAME_ID = \"{frame_id}\";</script>{scripts}</head>\n<body style=\"margin:0px;\">&#160;</body></html>"
        );
        let replacement = format!(
            "<div id=\"{holder_id}\" class=\"stack-iframe-holder\" style=\"width:500px;height:400px;display:none;\" data-frame-id=\"{frame_id}\" data-frame-title=\"{escaped_title}\" data-frame-scrolling=\"true\" data-frame-srcdoc=\"{srcdoc}\"><iframe id=\"{frame_id}\" class=\"stack-iframe\" style=\"width:100%;height:100%;border:0;\" scrolling=\"yes\" title=\"{escaped_title}\" referrerpolicy=\"no-referrer\" sandbox=\"allow-scripts allow-downloads\" srcdoc=\"{srcdoc}\"></iframe></div>",
            srcdoc = escape_html_attr(&srcdoc),
        );
        text.replace_range(m.start..m.end, &replacement);
        pos = m.start + replacement.len();
    }

    // 5c. Process [[jsxgraph ...]]...[[/jsxgraph]]
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "jsxgraph", pos) {
        *jxg_counter += 1;
        let idx = *jxg_counter;
        let div_id = format!("stack-jxg-{idx}");
        let frame_id = format!("stack-iframe-{idx}");
        let title = format!("STACK JSXGraph {idx}");
        let escaped_title = escape_html_attr(&title);
        let style_attr = compute_frame_style(&m.attrs, false);
        let (styles, scripts, clean_body) = extract_styles_and_scripts(&m.body);
        let refs = extract_input_refs(&m.attrs);

        let mut ref_preamble = String::new();
        for (inp_name, ref_var) in &refs {
            ref_preamble.push_str(&format!(
                "var {} = \"stack_input_{}\";\n",
                ref_var,
                escape_html_attr(inp_name)
            ));
        }
        let inline_code = if ref_preamble.is_empty() {
            clean_body.trim().to_string()
        } else {
            format!("{ref_preamble}{}", clean_body.trim())
        };

        let (imports, rest_body) = separate_js_imports(&clean_body);
        let css_href = extract_attr(&m.attrs, "overridecss")
            .map(|s| replace_cors_url(&s))
            .unwrap_or_else(|| "/corsscripts/jsxgraph.min.css".to_string());
        let js_src = extract_attr(&m.attrs, "overridejs")
            .map(|s| replace_cors_url(&s))
            .unwrap_or_else(|| "/corsscripts/jsxgraphcore.min.js".to_string());

        let wrapped_or_rest_body = if refs.is_empty() {
            rest_body
        } else {
            let promises = refs
                .iter()
                .map(|(inp, _)| format!("stack_js.request_access_to_input(\"{}\",true)", inp))
                .collect::<Vec<_>>()
                .join(",");
            let vars = refs
                .iter()
                .map(|(_, var)| var.as_str())
                .collect::<Vec<_>>()
                .join(",");
            format!("Promise.all([{promises}]).then(([{vars}]) => {{\n{rest_body}\n}});")
        };

        let srcdoc = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Strict//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\">\n<html xmlns=\"http://www.w3.org/1999/xhtml\" style=\"width:100%;height:100%;margin:0;padding:0;\">\n<head><title>{escaped_title}</title><style type=\"text/css\">\nhtml, body {{\n  width: 100%;\n  height: 100%;\n  margin: 0;\n  padding: 0;\n  overflow: hidden;\n}}\n.jxgbox, #jxgbox {{\n  width: 100%;\n  height: 100%;\n  box-sizing: border-box;\n}}\n</style><link rel=\"stylesheet\" type=\"text/css\" href=\"{escaped_css}\"/>{styles}<script>const FRAME_ID = \"{frame_id}\";</script><script type=\"text/javascript\" src=\"{escaped_js}\"></script>{scripts}</head>\n<body style=\"width:100%;height:100%;margin:0px;padding:0px;overflow:hidden;\"><div style=\"box-sizing:border-box;width:100%;height:100%;height:100vh;\"><div class=\"jxgbox\" id=\"jxgbox\" style=\"width:100%;height:100%;\"></div></div><script type=\"module\">\nimport stack_js from '/corsscripts/stackjsiframe.min.js';\nimport stack_jxg from '/corsscripts/stackjsxgraph.min.js';\n{imports}\nvar divid = \"jxgbox\";var BOARDID = divid;\n{wrapped_or_rest_body}\nfunction stackResizeBoard() {{\n  var b = (typeof JXG !== 'undefined' && JXG.boards) ? JXG.boards[divid] : null;\n  if (!b && typeof board !== 'undefined') {{ b = board; }}\n  if (b && b.resizeContainer) {{\n    var el = document.getElementById(divid);\n    if (el && el.clientHeight > 0 && (b.canvasHeight <= 0 || Math.abs(b.canvasHeight - el.clientHeight) > 2)) {{\n      b.resizeContainer(el.clientWidth, el.clientHeight);\n      b.fullUpdate();\n    }}\n  }}\n}}\nwindow.addEventListener('resize', stackResizeBoard);\nsetTimeout(stackResizeBoard, 20);\nsetTimeout(stackResizeBoard, 100);\nsetTimeout(stackResizeBoard, 300);\n</script></body></html>",
            escaped_css = escape_html_attr(&css_href),
            escaped_js = escape_html_attr(&js_src),
        );

        let replacement = format!(
            r#"<div id="{div_id}" class="jxgbox stack-jsxgraph stack-iframe-holder" style="{style_attr}" data-jxg-code="{code}" data-frame-id="{frame_id}" data-frame-title="{escaped_title}" data-frame-scrolling="false" data-frame-srcdoc="{srcdoc}"><iframe id="{frame_id}" class="stack-iframe" style="width:100%;height:100%;border:0;" scrolling="no" title="{escaped_title}" referrerpolicy="no-referrer" sandbox="allow-scripts allow-downloads" srcdoc="{srcdoc}"></iframe></div>"#,
            style_attr = escape_html_attr(&style_attr),
            code = escape_html_attr(&inline_code),
            srcdoc = escape_html_attr(&srcdoc),
        );
        text.replace_range(m.start..m.end, &replacement);
        pos = m.start + replacement.len();
    }

    // 5d. Replace any stray [[style]] or [[script]] blocks outside iframes with STACK error messages
    let style_err = " ERROR [[style]] blocks must be within iframes. ";
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "style", pos) {
        text.replace_range(m.start..m.end, style_err);
        pos = m.start + style_err.len();
    }
    let script_err = " ERROR [[script]] blocks must be within iframes. ";
    let mut pos = 0;
    while let Some(m) = find_castext_block(&text, "script", pos) {
        text.replace_range(m.start..m.end, script_err);
        pos = m.start + script_err.len();
    }

    // 6. Replace [[input:name]] and [[validation:name]]
    for inp in inputs {
        let input_tag = format!("[[input:{}]]", inp.name);
        if text.contains(&input_tag) {
            let widget_html = render_input_widget_html(inp, matrixparens);
            text = text.replace(&input_tag, &widget_html);
        }
        let val_tag = format!("[[validation:{}]]", inp.name);
        if text.contains(&val_tag) {
            let val_html = render_validation_badge_html(inp);
            text = text.replace(&val_tag, &val_html);
        }
    }

    // Also replace any remaining [[feedback:prtX]] with a subtle placeholder badge
    let mut out = String::with_capacity(text.len());
    let mut rest = text.as_str();
    while let Some(start) = rest.find("[[feedback:") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 11..];
        if let Some(end) = after.find("]]") {
            let prt_name = &after[..end];
            out.push_str(&format!(
                r#"<span class="stack-feedback-tag" data-prt="{prt}">PRT: {prt}</span>"#,
                prt = escape_html_attr(prt_name)
            ));
            rest = &after[end + 2..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        }
    }
    out.push_str(rest);

    restore_castext_escapes(out, &escapes)
}

fn input_type_str(it: InputType) -> &'static str {
    match it {
        InputType::Algebraic => "algebraic",
        InputType::Equiv => "equiv",
        InputType::GeoGebra => "geogebra",
        InputType::JSON => "json",
        InputType::Numerical => "numerical",
        InputType::Matrix => "matrix",
        InputType::Varmatrix => "varmatrix",
        InputType::String => "string",
        InputType::Notes => "notes",
        InputType::Boolean => "boolean",
        InputType::Parsons => "parsons",
        InputType::Dropdown => "dropdown",
        InputType::Radio => "radio",
        InputType::Textarea => "textarea",
        InputType::Checkbox => "checkbox",
        InputType::Units => "units",
        InputType::Singlechar => "singlechar",
    }
}

fn render_input_widget_html(inp: &EvaluatedInput, matrixparens: MatrixParens) -> String {
    let name_esc = escape_html_attr(&inp.name);
    let tans_esc = escape_html_attr(&inp.tans_value);
    let type_str = input_type_str(inp.input_type);

    match inp.input_type {
        InputType::Matrix | InputType::Varmatrix => {
            if let Some(rows) = &inp.matrix_cells {
                let (l_delim, r_delim) = match matrixparens {
                    MatrixParens::Square => ("bracket-square-l", "bracket-square-r"),
                    MatrixParens::Round => ("bracket-round-l", "bracket-round-r"),
                    MatrixParens::Curly => ("bracket-curly-l", "bracket-curly-r"),
                    MatrixParens::Bar => ("bracket-bar-l", "bracket-bar-r"),
                    MatrixParens::None => ("bracket-none", "bracket-none"),
                };
                let mut grid = format!(
                    r#"<span class="stack-matrix-widget {l_delim} {r_delim}" data-input-name="{name_esc}">"#,
                );
                grid.push_str(r#"<table class="stack-matrix-table"><tbody>"#);
                for (r_idx, row) in rows.iter().enumerate() {
                    grid.push_str("<tr>");
                    for (c_idx, cell_val) in row.iter().enumerate() {
                        let cell_esc = escape_html_attr(cell_val);
                        let sz = inp.boxsize.clamp(2, 12);
                        grid.push_str(&format!(
                            r#"<td><input type="text" class="stack-input-control stack-matrix-cell" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-row="{r_idx}" data-col="{c_idx}" data-tans="{cell_esc}" size="{sz}" aria-label="{name_esc}[{r},{c}]"/></td>"#,
                            r = r_idx + 1,
                            c = c_idx + 1
                        ));
                    }
                    grid.push_str("</tr>");
                }
                grid.push_str("</tbody></table></span>");
                return grid;
            }
            // Fallback if matrix dimensions could not be evaluated
            format!(
                r#"<textarea id="stack-input-{name_esc}" class="stack-input-control stack-input-textarea" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-tans="{tans_esc}" rows="3" cols="20" placeholder="matrix ({name_esc})"></textarea>"#
            )
        }
        InputType::Boolean => {
            format!(
                r#"<select id="stack-input-{name_esc}" class="stack-input-control stack-input-select" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-tans="{tans_esc}">
                    <option value="">Ei vastausta</option>
                    <option value="true">Tosi</option>
                    <option value="false">Epätosi</option>
                </select>"#
            )
        }
        InputType::Dropdown => {
            if let Some(choices) = &inp.choices {
                let correct_val = choices
                    .iter()
                    .find(|c| c.correct)
                    .map(|c| escape_html_attr(&c.value))
                    .unwrap_or_default();
                let mut html = format!(
                    r#"<select id="stack-input-{name_esc}" class="stack-input-control stack-input-select" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-tans="{correct_val}">
                        <option value="">Valitse</option>"#
                );
                for ch in choices {
                    let val_esc = escape_html_attr(&ch.value);
                    let disp = strip_tags_for_option(&ch.display_html);
                    html.push_str(&format!(
                        r#"<option value="{val_esc}">{disp}</option>"#,
                        disp = escape_html_attr(&disp)
                    ));
                }
                html.push_str("</select>");
                return html;
            }
            format!(
                r#"<select id="stack-input-{name_esc}" class="stack-input-control stack-input-select" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-tans="{tans_esc}">
                    <option value="">({name_esc})</option>
                </select>"#
            )
        }
        InputType::Checkbox | InputType::Radio => {
            let input_kind = if inp.input_type == InputType::Checkbox {
                "checkbox"
            } else {
                "radio"
            };
            if let Some(choices) = &inp.choices {
                let mut html =
                    format!(r#"<div class="stack-choices-group" data-input-name="{name_esc}">"#);
                for (idx, ch) in choices.iter().enumerate() {
                    let val_esc = escape_html_attr(&ch.value);
                    let corr_attr = if ch.correct { "true" } else { "false" };
                    html.push_str(&format!(
                        r#"<label class="stack-choice-item" data-correct="{corr_attr}">
                            <input type="{input_kind}" class="stack-input-control stack-input-choice" name="stack_{name_esc}" value="{val_esc}" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-choice-idx="{idx}" data-correct="{corr_attr}"/>
                            <span class="stack-choice-label">{disp}</span>
                        </label>"#,
                        disp = ch.display_html
                    ));
                }
                html.push_str("</div>");
                return html;
            }
            format!(
                r#"<input id="stack-input-{name_esc}" type="text" class="stack-input-control stack-input-text" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-tans="{tans_esc}" size="{sz}"/>"#,
                sz = inp.boxsize.max(5)
            )
        }
        InputType::Textarea | InputType::Equiv | InputType::Notes => {
            let hint_esc = escape_html_attr(&inp.syntaxhint);
            format!(
                r#"<textarea id="stack-input-{name_esc}" class="stack-input-control stack-input-textarea" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-tans="{tans_esc}" rows="4" cols="{cols}" placeholder="{hint_esc}"></textarea>"#,
                cols = inp.boxsize.clamp(15, 60)
            )
        }
        _ => {
            let sz = inp.boxsize.clamp(4, 60);
            let hint_esc = escape_html_attr(&inp.syntaxhint);
            let (val_attr, placeholder_attr) = match inp.syntaxattribute {
                Some(SyntaxAttribute::Placeholder) => (String::new(), hint_esc),
                _ => (hint_esc, inp.name.clone()),
            };
            format!(
                r#"<input id="stack-input-{name_esc}" type="text" class="stack-input-control stack-input-text" data-input-name="{name_esc}" data-stack-input-type="{type_str}" data-tans="{tans_esc}" size="{sz}" value="{val_attr}" placeholder="{placeholder_attr}"/>"#
            )
        }
    }
}

fn strip_tags_for_option(s: &str) -> String {
    s.replace("\\(", "").replace("\\)", "")
}

fn render_validation_badge_html(inp: &EvaluatedInput) -> String {
    let name_esc = escape_html_attr(&inp.name);
    let latex = if inp.tans_latex.is_empty() {
        escape_html_attr(&inp.tans_value)
    } else {
        format!("\\({}\\)", inp.tans_latex)
    };
    format!(
        r#"<span class="stack-validation-slot" data-validation-for="{name_esc}" title="Mallivastaus {name_esc}"><span class="stack-validation-label">{name_esc} = </span><span class="stack-validation-math">{latex}</span></span>"#
    )
}

/// Pure-Rust static fallback renderer for `StackQuestion` (used when Maxima is unavailable).
pub fn render_stack_question_static(q: &StackQuestion, seed: u32) -> EvaluatedStackQuestion {
    let tex_map = BTreeMap::new();
    let str_map = BTreeMap::new();
    let if_map = BTreeMap::new();

    let inputs: Vec<EvaluatedInput> = q
        .input
        .iter()
        .map(|inp| EvaluatedInput {
            name: inp.name.clone(),
            input_type: inp.input_type,
            boxsize: inp.boxsize,
            syntaxhint: inp.syntaxhint.clone(),
            syntaxattribute: inp.syntaxattribute,
            tans_expr: inp.tans.clone(),
            tans_value: inp.tans.clone(),
            tans_latex: format!("\\mathtt{{{}}}", inp.tans.replace('_', "\\_")),
            matrix_cells: None,
            choices: None,
        })
        .collect();

    let mut jxg_counter = 0;
    let questiontext_html = render_castext(
        &q.questiontext,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let generalfeedback_html = render_castext(
        &q.generalfeedback,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let specificfeedback_html = render_castext(
        &q.specificfeedback,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let note_ft = FormattedText {
        format: TextFormat::Html,
        text: q.questionnote.text.clone(),
        file: Vec::new(),
    };
    let questionnote_html = render_castext(
        &note_ft,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let questiondescription_html = q.questiondescription.as_ref().map(|ft| {
        render_castext(
            ft,
            &tex_map,
            &str_map,
            &if_map,
            &inputs,
            q.matrixparens,
            &mut jxg_counter,
        )
    });
    let hint_html = q.hint.as_ref().map(|ft| {
        render_castext(
            ft,
            &tex_map,
            &str_map,
            &if_map,
            &inputs,
            q.matrixparens,
            &mut jxg_counter,
        )
    });
    let prtcorrect_html = render_castext(
        &q.prtcorrect,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let prtpartiallycorrect_html = render_castext(
        &q.prtpartiallycorrect,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let prtincorrect_html = render_castext(
        &q.prtincorrect,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );

    let prts = q
        .prt
        .iter()
        .map(|p| EvaluatedPrt {
            name: p.name.clone(),
            value: p.value,
            autosimplify: p.autosimplify,
            feedbackvariables: p.feedbackvariables.text.clone(),
            nodes: p
                .node
                .iter()
                .map(|n| EvaluatedNode {
                    name: n.name.clone(),
                    description: n.description.clone(),
                    answertest: format!("{:?}", n.answertest),
                    sans: n.sans.clone(),
                    tans: n.tans.clone(),
                    testoptions: n.testoptions.clone(),
                    quiet: n.quiet,
                    truescoremode: score_mode_str(n.truescoremode).to_string(),
                    truescore: n.truescore,
                    truenextnode: n.truenextnode,
                    trueanswernote: n.trueanswernote.clone(),
                    truefeedback_html: render_castext(
                        &n.truefeedback,
                        &tex_map,
                        &str_map,
                        &if_map,
                        &inputs,
                        q.matrixparens,
                        &mut jxg_counter,
                    ),
                    falsescoremode: score_mode_str(n.falsescoremode).to_string(),
                    falsescore: n.falsescore,
                    falsenextnode: n.falsenextnode,
                    falseanswernote: n.falseanswernote.clone(),
                    falsefeedback_html: render_castext(
                        &n.falsefeedback,
                        &tex_map,
                        &str_map,
                        &if_map,
                        &inputs,
                        q.matrixparens,
                        &mut jxg_counter,
                    ),
                })
                .collect(),
        })
        .collect();

    EvaluatedStackQuestion {
        seed,
        maxima_ok: false,
        maxima_error: None,
        questiontext_html,
        generalfeedback_html,
        specificfeedback_html,
        questionnote_html,
        questiondescription_html,
        hint_html,
        prtcorrect_html,
        prtpartiallycorrect_html,
        prtincorrect_html,
        inputs,
        variables: Vec::new(),
        prts,
    }
}

fn score_mode_str(sm: ScoreMode) -> &'static str {
    match sm {
        ScoreMode::Equals => "=",
        ScoreMode::Plus => "+",
        ScoreMode::Minus => "-",
    }
}

/// Format answer numbering label (`a.`, `A.`, `1.`, `i.`, `I.`) for Multichoice questions.
pub fn format_answer_number(numbering: AnswerNumbering, idx: usize) -> String {
    match numbering {
        AnswerNumbering::ABC => {
            let ch = (b'a' + (idx as u8 % 26)) as char;
            format!("{ch}. ")
        }
        AnswerNumbering::ABCD => {
            let ch = (b'A' + (idx as u8 % 26)) as char;
            format!("{ch}. ")
        }
        AnswerNumbering::Number => format!("{}. ", idx + 1),
        AnswerNumbering::III => format!("{}. ", to_roman(idx + 1).to_ascii_lowercase()),
        AnswerNumbering::IIII => format!("{}. ", to_roman(idx + 1)),
        AnswerNumbering::None => String::new(),
    }
}

fn to_roman(mut n: usize) -> String {
    let numerals = [
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for &(val, sym) in &numerals {
        while n >= val {
            out.push_str(sym);
            n -= val;
        }
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
pub fn evaluate_stack_question_maxima(q: &StackQuestion, seed: u32) -> EvaluatedStackQuestion {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let statements = normalize_maxima_statements(&q.questionvariables.text);
    let assigned_vars = extract_assigned_variables(&statements);

    // Collect expressions from all text fields
    let mut tex_exprs = BTreeSet::new();
    let mut str_exprs = BTreeSet::new();
    let mut if_exprs = BTreeSet::new();

    for txt in [
        &q.questiontext.text,
        &q.generalfeedback.text,
        &q.specificfeedback.text,
        &q.questionnote.text,
        &q.prtcorrect.text,
        &q.prtpartiallycorrect.text,
        &q.prtincorrect.text,
    ] {
        collect_castext_expressions(txt, &mut tex_exprs, &mut str_exprs, &mut if_exprs);
    }
    if let Some(desc) = &q.questiondescription {
        collect_castext_expressions(&desc.text, &mut tex_exprs, &mut str_exprs, &mut if_exprs);
    }
    if let Some(h) = &q.hint {
        collect_castext_expressions(&h.text, &mut tex_exprs, &mut str_exprs, &mut if_exprs);
    }
    for prt in &q.prt {
        for node in &prt.node {
            collect_castext_expressions(
                &node.truefeedback.text,
                &mut tex_exprs,
                &mut str_exprs,
                &mut if_exprs,
            );
            collect_castext_expressions(
                &node.falsefeedback.text,
                &mut tex_exprs,
                &mut str_exprs,
                &mut if_exprs,
            );
        }
    }

    let tex_list: Vec<String> = tex_exprs.into_iter().collect();
    let str_list: Vec<String> = str_exprs.into_iter().collect();
    let if_list: Vec<String> = if_exprs.into_iter().collect();

    // Build Maxima batch script
    let mut script = String::new();
    script.push_str("linel: 10000$\n");
    script.push_str("display2d: false$\n");
    script.push_str("ratprint: false$\n");
    script.push_str("fpprintprec: 12$\n");
    script.push_str(&format!(
        "set_random_state(make_random_state({}))$\n",
        seed.max(1)
    ));
    script.push_str(&format!(
        "simp: {}$\n",
        if q.questionsimplify { "true" } else { "false" }
    ));
    if q.assumepositive {
        script.push_str("assume_pos: true$\n");
    }
    script.push_str(&format!(
        "sqrtdispflag: {}$\n",
        if q.sqrtsign { "true" } else { "false" }
    ));
    match q.inversetrig {
        InverseTrig::CosMinus1 => {
            script.push_str(
                "texput(acos, \"\\\\cos^{-1}\", prefix)$ texput(asin, \"\\\\sin^{-1}\", prefix)$ texput(atan, \"\\\\tan^{-1}\", prefix)$\n",
            );
        }
        InverseTrig::Acos => {
            script.push_str(
                "texput(acos, \"\\\\mathrm{acos}\", prefix)$ texput(asin, \"\\\\mathrm{asin}\", prefix)$ texput(atan, \"\\\\mathrm{atan}\", prefix)$\n",
            );
        }
        InverseTrig::Arccos | InverseTrig::ArccosArcosh => {
            script.push_str(
                "texput(acos, \"\\\\arccos\", prefix)$ texput(asin, \"\\\\arcsin\", prefix)$ texput(atan, \"\\\\arctan\", prefix)$\n",
            );
        }
    }
    match q.complexno {
        ComplexNo::I => script.push_str("texput(%i, \"\\\\mathrm{i}\")$\n"),
        ComplexNo::J => script.push_str("texput(%i, \"\\\\mathrm{j}\")$\n"),
        ComplexNo::SymI => script.push_str("texput(%i, \"i\")$\n"),
        ComplexNo::SymJ => script.push_str("texput(%i, \"j\")$\n"),
    }

    // STACK standard functions
    script.push_str(
        r#"
ln(x) := log(x)$
lg(x) := log(x)/log(10)$
log10(x) := log(x)/log(10)$
log2(x) := log(x)/log(2)$
cosec(x) := csc(x)$
totient(n) := eulerphi(n)$
texput(log, "\\ln", prefix)$
matrixnorm(A, [p]) := sqrt(apply("+", map(lambda([x], x^2), list_matrix_entries(A))))$
matrix_size(m) := if matrixp(m) then [length(m), if length(m) > 0 then length(m[1]) else 0] else [0, 0]$
comp_square(ex, var) := block([a, b, c], a: coeff(expand(ex), var, 2), b: coeff(expand(ex), var, 1), c: coeff(expand(ex), var, 0), if a = 0 then ex else a*(var + b/(2*a))^2 + c - b^2/(4*a))$
decimalplaces(x, n) := float(round(x * 10^n) / 10^n)$
significantfigures(x, n) := block([f, d], if x = 0 then return(0), f: float(abs(x)), d: floor(log(f)/log(10)), float(round(x * 10^(n - 1 - d)) / 10^(n - 1 - d)))$
dispdp(x, n) := decimalplaces(x, n)$
dispsf(x, n) := significantfigures(x, n)$
scientific_notation(x, [n]) := float(x)$
rand(ex) := block(
  if integerp(ex) then return(if ex > 0 then random(ex) else 0),
  if floatnump(ex) then return(if ex > 0.0 then random(ex) else 0.0),
  if listp(ex) then return(if emptyp(ex) then [] else ex[random(length(ex))+1]),
  if setp(ex) then return(if emptyp(ex) then {} else listify(ex)[random(length(ex))+1]),
  if matrixp(ex) then return(matrixmap(rand, ex)),
  return(ex)
)$
rand_with_step(lower, upper, st) := block([n_steps],
  if st <= 0 or lower > upper then return(lower),
  n_steps: floor((upper - lower) / st),
  return(lower + st * random(n_steps + 1))
)$
rand_with_prohib(lower, upper, prohib) := block([avail],
  avail: listify(setdifference(setify(makelist(i, i, lower, upper)), setify(if listp(prohib) then prohib else [prohib]))),
  if emptyp(avail) then return(lower),
  return(rand(avail))
)$
rand_selection(ex, n) := block([lst, res, idx, k],
  lst: if setp(ex) then listify(ex) else if listp(ex) then copylist(ex) else [ex],
  res: [],
  for k: 1 thru min(n, length(lst)) do (
    idx: random(length(lst)) + 1,
    res: endcons(lst[idx], res),
    lst: append(rest(lst, -(length(lst) - idx + 1)), rest(lst, idx))
  ),
  return(res)
)$
stack_eval_tex(idx, ex) := block([s, t],
  s: if stringp(ex) then "1" else "0",
  t: if stringp(ex) then ex else tex1(ex),
  print(sconcat("__STACK_TEX_", idx, "__|", s, "|", t, "|__END__"))
)$
stack_eval_str(idx, ex) := block([v],
  v: if stringp(ex) then ex else string(ex),
  print(sconcat("__STACK_STR_", idx, "__|", v, "|__END__"))
)$
stack_eval_if(idx, ex) := block([b],
  b: is(ex),
  print(sconcat("__STACK_IF_", idx, "__|", if b = true then "true" else "false", "|__END__"))
)$
stack_eval_input(idx, ex) := block([r, c, i, j, item, v, corr, disp],
  if matrixp(ex) then (
    r: matrix_size(ex)[1],
    c: matrix_size(ex)[2],
    print(sconcat("__STACK_INP_", idx, "__|MATRIX|", r, "|", c, "|", string(ex), "|", tex1(ex), "|__END__")),
    for i: 1 thru r do (
      for j: 1 thru c do (
        print(sconcat("__STACK_MCELL_", idx, "_", i, "_", j, "__|", string(ex[i, j]), "|__END__"))
      )
    )
  ) else if listp(ex) and not emptyp(ex) and listp(first(ex)) then (
    print(sconcat("__STACK_INP_", idx, "__|CHOICES|", string(ex), "|", tex1(ex), "|__END__")),
    for i: 1 thru length(ex) do (
      item: ex[i],
      v: if length(item) >= 1 then (if stringp(item[1]) then item[1] else string(item[1])) else "",
      corr: if length(item) >= 2 and is(item[2] = true) then "true" else "false",
      disp: if length(item) >= 3 then (if stringp(item[3]) then item[3] else sconcat("\\(", tex1(item[3]), "\\)")) else (if length(item) >= 1 then sconcat("\\(", tex1(item[1]), "\\)") else ""),
      print(sconcat("__STACK_CHOICE_", idx, "_", i, "__|", v, "|", corr, "|", disp, "|__END__"))
    )
  ) else (
    print(sconcat("__STACK_INP_", idx, "__|SCALAR|", if stringp(ex) then ex else string(ex), "|", if stringp(ex) then ex else tex1(ex), "|__END__"))
  )
)$
stack_bind_input(ex) := block([i, item],
  if listp(ex) and not emptyp(ex) and listp(first(ex)) then (
    for i: 1 thru length(ex) do (
      item: ex[i],
      if length(item) >= 2 and is(item[2] = true) then return(item[1])
    ),
    return(first(first(ex)))
  ),
  return(ex)
)$
stack_eval_var(name_str, ex) := block(
  print(sconcat("__STACK_VAR__|", name_str, "|", if stringp(ex) then ex else string(ex), "|", if stringp(ex) then ex else tex1(ex), "|__END__"))
)$
"#,
    );

    // Question variables
    for stmt in &statements {
        script.push_str(stmt);
        script.push_str("$\n");
    }

    // Evaluate inputs first and also bind input variable names to their model answers for feedback evaluation
    for (idx, inp) in q.input.iter().enumerate() {
        if !inp.tans.trim().is_empty() {
            script.push_str(&format!(
                "errcatch(stack_eval_input({}, ({})))$\n",
                idx, inp.tans
            ));
            script.push_str(&format!(
                "errcatch({}: stack_bind_input({}))$\n",
                inp.name, inp.tans
            ));
        }
    }

    // Also run PRT feedbackvariables in errcatch so any feedback expressions referencing them work
    for prt in &q.prt {
        for stmt in normalize_maxima_statements(&prt.feedbackvariables.text) {
            script.push_str(&format!("errcatch({})$\n", stmt));
        }
    }

    // Evaluate assigned variables
    for v in &assigned_vars {
        script.push_str(&format!("errcatch(stack_eval_var(\"{v}\", {v}))$\n"));
    }

    // Evaluate CASText expressions
    for (idx, expr) in tex_list.iter().enumerate() {
        script.push_str(&format!("errcatch(stack_eval_tex({}, ({})))$\n", idx, expr));
    }
    for (idx, expr) in str_list.iter().enumerate() {
        script.push_str(&format!("errcatch(stack_eval_str({}, ({})))$\n", idx, expr));
    }
    for (idx, expr) in if_list.iter().enumerate() {
        script.push_str(&format!("errcatch(stack_eval_if({}, ({})))$\n", idx, expr));
    }
    script.push_str("print(\"__STACK_DONE__\")$\n");

    let child = Command::new("maxima")
        .arg("--very-quiet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let Ok(mut child) = child else {
        let mut fallback = render_stack_question_static(q, seed);
        fallback.maxima_error = Some("Failed to spawn /usr/bin/maxima".to_string());
        return fallback;
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(script.as_bytes());
    }

    let Ok(output) = child.wait_with_output() else {
        let mut fallback = render_stack_question_static(q, seed);
        fallback.maxima_error = Some("Maxima process failed".to_string());
        return fallback;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let maxima_ok = stdout.contains("__STACK_DONE__");

    let mut tex_map: BTreeMap<String, (bool, String)> = BTreeMap::new();
    let mut str_map: BTreeMap<String, String> = BTreeMap::new();
    let mut if_map: BTreeMap<String, bool> = BTreeMap::new();
    let mut variables: Vec<EvaluatedVariable> = Vec::new();

    let mut inp_scalar: BTreeMap<usize, (String, String)> = BTreeMap::new();
    let mut inp_matrix_dims: BTreeMap<usize, (usize, usize, String, String)> = BTreeMap::new();
    let mut inp_matrix_cells: BTreeMap<(usize, usize, usize), String> = BTreeMap::new();
    let mut inp_choices: BTreeMap<usize, Vec<EvaluatedChoice>> = BTreeMap::new();

    for raw_line in stdout.lines() {
        let line = raw_line.trim();
        let Some(end_idx) = line.rfind("|__END__") else {
            continue;
        };
        let content = &line[..end_idx];

        if let Some(rest) = content.strip_prefix("__STACK_TEX_") {
            if let Some((idx_str, payload)) = rest.split_once("__")
                && let Ok(idx) = idx_str.parse::<usize>()
                && let Some(payload_inner) = payload.strip_prefix('|')
                && let Some((s_flag, tex_val)) = payload_inner.split_once('|')
                && let Some(expr) = tex_list.get(idx)
            {
                tex_map.insert(expr.clone(), (s_flag == "1", tex_val.to_string()));
            }
        } else if let Some(rest) = content.strip_prefix("__STACK_STR_") {
            if let Some((idx_str, payload)) = rest.split_once("__")
                && let Ok(idx) = idx_str.parse::<usize>()
                && let Some(val) = payload.strip_prefix('|')
                && let Some(expr) = str_list.get(idx)
            {
                str_map.insert(expr.clone(), val.to_string());
            }
        } else if let Some(rest) = content.strip_prefix("__STACK_IF_") {
            if let Some((idx_str, payload)) = rest.split_once("__")
                && let Ok(idx) = idx_str.parse::<usize>()
                && let Some(val) = payload.strip_prefix('|')
                && let Some(expr) = if_list.get(idx)
            {
                if_map.insert(expr.clone(), val == "true");
            }
        } else if let Some(rest) = content.strip_prefix("__STACK_VAR__|") {
            let parts: Vec<&str> = rest.splitn(3, '|').collect();
            if parts.len() == 3 {
                variables.push(EvaluatedVariable {
                    name: parts[0].to_string(),
                    value: parts[1].to_string(),
                    latex: clean_maxima_tex(parts[2], q.matrixparens),
                });
            }
        } else if let Some(rest) = content.strip_prefix("__STACK_INP_") {
            if let Some((idx_str, payload)) = rest.split_once("__")
                && let Ok(idx) = idx_str.parse::<usize>()
                && let Some(payload_inner) = payload.strip_prefix('|')
            {
                if let Some(m_rest) = payload_inner.strip_prefix("MATRIX|") {
                    let parts: Vec<&str> = m_rest.splitn(4, '|').collect();
                    if parts.len() == 4
                        && let (Ok(r), Ok(c)) = (parts[0].parse(), parts[1].parse())
                    {
                        inp_matrix_dims.insert(
                            idx,
                            (
                                r,
                                c,
                                parts[2].to_string(),
                                clean_maxima_tex(parts[3], q.matrixparens),
                            ),
                        );
                    }
                } else if let Some(c_rest) = payload_inner.strip_prefix("CHOICES|") {
                    let parts: Vec<&str> = c_rest.splitn(2, '|').collect();
                    if parts.len() == 2 {
                        inp_scalar.insert(
                            idx,
                            (
                                parts[0].to_string(),
                                clean_maxima_tex(parts[1], q.matrixparens),
                            ),
                        );
                    }
                } else if let Some(s_rest) = payload_inner.strip_prefix("SCALAR|") {
                    let parts: Vec<&str> = s_rest.splitn(2, '|').collect();
                    if parts.len() == 2 {
                        inp_scalar.insert(
                            idx,
                            (
                                parts[0].to_string(),
                                clean_maxima_tex(parts[1], q.matrixparens),
                            ),
                        );
                    }
                }
            }
        } else if let Some(rest) = content.strip_prefix("__STACK_MCELL_") {
            if let Some((coords, payload)) = rest.split_once("__")
                && let Some(val) = payload.strip_prefix('|')
            {
                let nums: Vec<usize> = coords
                    .split('_')
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect();
                if nums.len() == 3 {
                    inp_matrix_cells.insert((nums[0], nums[1], nums[2]), val.to_string());
                }
            }
        } else if let Some(rest) = content.strip_prefix("__STACK_CHOICE_")
            && let Some((coords, payload)) = rest.split_once("__")
            && let Some(payload_inner) = payload.strip_prefix('|')
        {
            let nums: Vec<usize> = coords
                .split('_')
                .filter_map(|s| s.parse::<usize>().ok())
                .collect();
            let parts: Vec<&str> = payload_inner.splitn(3, '|').collect();
            if nums.len() == 2 && parts.len() == 3 {
                inp_choices
                    .entry(nums[0])
                    .or_default()
                    .push(EvaluatedChoice {
                        value: parts[0].to_string(),
                        correct: parts[1] == "true",
                        display_html: parts[2].to_string(),
                    });
            }
        }
    }

    let inputs: Vec<EvaluatedInput> = q
        .input
        .iter()
        .enumerate()
        .map(|(idx, inp)| {
            if let Some((r, c, m_str, m_tex)) = inp_matrix_dims.get(&idx) {
                let mut rows = Vec::with_capacity(*r);
                for i in 1..=*r {
                    let mut row = Vec::with_capacity(*c);
                    for j in 1..=*c {
                        let cell = inp_matrix_cells
                            .get(&(idx, i, j))
                            .cloned()
                            .unwrap_or_default();
                        row.push(cell);
                    }
                    rows.push(row);
                }
                EvaluatedInput {
                    name: inp.name.clone(),
                    input_type: inp.input_type,
                    boxsize: inp.boxsize,
                    syntaxhint: inp.syntaxhint.clone(),
                    syntaxattribute: inp.syntaxattribute,
                    tans_expr: inp.tans.clone(),
                    tans_value: m_str.clone(),
                    tans_latex: m_tex.clone(),
                    matrix_cells: Some(rows),
                    choices: None,
                }
            } else {
                let (val, tex) = inp_scalar
                    .get(&idx)
                    .cloned()
                    .unwrap_or_else(|| (inp.tans.clone(), inp.tans.clone()));
                let choices = inp_choices.remove(&idx);
                EvaluatedInput {
                    name: inp.name.clone(),
                    input_type: inp.input_type,
                    boxsize: inp.boxsize,
                    syntaxhint: inp.syntaxhint.clone(),
                    syntaxattribute: inp.syntaxattribute,
                    tans_expr: inp.tans.clone(),
                    tans_value: val,
                    tans_latex: tex,
                    matrix_cells: None,
                    choices,
                }
            }
        })
        .collect();

    let mut jxg_counter = 0;
    let questiontext_html = render_castext(
        &q.questiontext,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let generalfeedback_html = render_castext(
        &q.generalfeedback,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let specificfeedback_html = render_castext(
        &q.specificfeedback,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let note_ft = FormattedText {
        format: TextFormat::Html,
        text: q.questionnote.text.clone(),
        file: Vec::new(),
    };
    let questionnote_html = render_castext(
        &note_ft,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let questiondescription_html = q.questiondescription.as_ref().map(|ft| {
        render_castext(
            ft,
            &tex_map,
            &str_map,
            &if_map,
            &inputs,
            q.matrixparens,
            &mut jxg_counter,
        )
    });
    let hint_html = q.hint.as_ref().map(|ft| {
        render_castext(
            ft,
            &tex_map,
            &str_map,
            &if_map,
            &inputs,
            q.matrixparens,
            &mut jxg_counter,
        )
    });
    let prtcorrect_html = render_castext(
        &q.prtcorrect,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let prtpartiallycorrect_html = render_castext(
        &q.prtpartiallycorrect,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );
    let prtincorrect_html = render_castext(
        &q.prtincorrect,
        &tex_map,
        &str_map,
        &if_map,
        &inputs,
        q.matrixparens,
        &mut jxg_counter,
    );

    let prts = q
        .prt
        .iter()
        .map(|p| EvaluatedPrt {
            name: p.name.clone(),
            value: p.value,
            autosimplify: p.autosimplify,
            feedbackvariables: p.feedbackvariables.text.clone(),
            nodes: p
                .node
                .iter()
                .map(|n| EvaluatedNode {
                    name: n.name.clone(),
                    description: n.description.clone(),
                    answertest: format!("{:?}", n.answertest),
                    sans: n.sans.clone(),
                    tans: n.tans.clone(),
                    testoptions: n.testoptions.clone(),
                    quiet: n.quiet,
                    truescoremode: score_mode_str(n.truescoremode).to_string(),
                    truescore: n.truescore,
                    truenextnode: n.truenextnode,
                    trueanswernote: n.trueanswernote.clone(),
                    truefeedback_html: render_castext(
                        &n.truefeedback,
                        &tex_map,
                        &str_map,
                        &if_map,
                        &inputs,
                        q.matrixparens,
                        &mut jxg_counter,
                    ),
                    falsescoremode: score_mode_str(n.falsescoremode).to_string(),
                    falsescore: n.falsescore,
                    falsenextnode: n.falsenextnode,
                    falseanswernote: n.falseanswernote.clone(),
                    falsefeedback_html: render_castext(
                        &n.falsefeedback,
                        &tex_map,
                        &str_map,
                        &if_map,
                        &inputs,
                        q.matrixparens,
                        &mut jxg_counter,
                    ),
                })
                .collect(),
        })
        .collect();

    EvaluatedStackQuestion {
        seed,
        maxima_ok,
        maxima_error: if maxima_ok {
            None
        } else {
            Some("Maxima evaluation did not complete cleanly".to_string())
        },
        questiontext_html,
        generalfeedback_html,
        specificfeedback_html,
        questionnote_html,
        questiondescription_html,
        hint_html,
        prtcorrect_html,
        prtpartiallycorrect_html,
        prtincorrect_html,
        inputs,
        variables,
        prts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_eval_all_formats() {
        let test_xml = include_str!(
            "../../../material/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723.xml"
        );
        let quiz = parse_moodle_xml(test_xml).expect("should parse Discrete Mathematics XML");
        assert!(!quiz.question.is_empty());

        if let Some(Question::Stack(sq)) = quiz
            .question
            .iter()
            .find(|q| matches!(q, Question::Stack(_)))
        {
            let ev = evaluate_stack_question_maxima(sq, 42);
            assert!(ev.maxima_ok);
            assert!(!ev.variables.is_empty());
            assert!(!ev.inputs.is_empty());
        }

        let simple_xml = include_str!("../../../material/STACK_tehtavat/stack_linear.xml");
        let simple_quiz = parse_moodle_xml(simple_xml).expect("should parse simplified STACK XML");
        assert_eq!(simple_quiz.question.len(), 1);
        if let Question::Stack(sq) = &simple_quiz.question[0] {
            let ev = evaluate_stack_question_maxima(sq, 123);
            assert!(ev.maxima_ok);
            assert_eq!(ev.inputs.len(), 1);
        }

        let exported = export_quiz_to_moodle_xml(&quiz).expect("should export quiz to XML");
        assert!(exported.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<quiz>"));
        let reparsed = parse_moodle_xml(&exported).expect("should reparse exported XML");
        assert_eq!(quiz, reparsed);
    }

    fn make_ft(text: &str) -> FormattedText {
        FormattedText {
            format: TextFormat::Html,
            text: text.to_string(),
            file: Vec::new(),
        }
    }

    #[test]
    fn test_johdatus_graafiteoriaan_jsxgraph_rendering() {
        let xml = include_str!(
            "../../../material/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723/johdatus_graafiteoriaan.xml"
        );
        let quiz = parse_moodle_xml(xml).unwrap();
        for q in &quiz.question {
            if let Question::Stack(sq) = q {
                let ev = evaluate_stack_question_maxima(sq, 42);
                assert!(
                    ev.questiontext_html.contains("class=\"jxgbox stack-jsxgraph stack-iframe-holder\""),
                    "should contain jsxgraph iframe holder"
                );
                assert!(
                    ev.questiontext_html.contains("<iframe id=\"stack-iframe-"),
                    "should render isolated iframe"
                );
                assert!(
                    ev.questiontext_html.contains("sandbox=\"allow-scripts allow-downloads\""),
                    "iframe must have sandbox without allow-same-origin"
                );
                assert!(
                    !ev.questiontext_html.contains("allow-same-origin"),
                    "must not contain allow-same-origin"
                );
                assert!(
                    ev.questiontext_html.contains("var coord = [["),
                    "matrix_size should properly evaluate coordinates for graph nodes"
                );
                break;
            }
        }
    }

    #[test]
    fn test_stack_js_comment_and_escape() {
        let raw = "before [[comment]]{@bad_cas@} [[comment]]nested[[/comment]][[/comment]] middle [[escape]][[javascript]]][[/escape]] and [[escape value=\"{@also_escaped@}\"/]] after";
        let mut tex_exprs = BTreeSet::new();
        let mut str_exprs = BTreeSet::new();
        let mut if_exprs = BTreeSet::new();
        collect_castext_expressions(raw, &mut tex_exprs, &mut str_exprs, &mut if_exprs);
        assert!(
            !tex_exprs.contains("bad_cas"),
            "commented expression must not be collected"
        );
        assert!(
            !tex_exprs.contains("also_escaped"),
            "escaped expression must not be collected"
        );

        let mut counter = 0;
        let rendered = render_castext(
            &make_ft(raw),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            MatrixParens::Square,
            &mut counter,
        );
        assert_eq!(
            rendered,
            "before  middle [[javascript]]] and {@also_escaped@} after"
        );
    }

    #[test]
    fn test_stack_js_template_quid_cors_jsstring_entityescape() {
        let raw = concat!(
            "[[template name=\"board\"]]custom();[[/template]]",
            "[[template mode=\"default\" name=\"board\"]]default();[[/template]]|",
            "[[template mode=\"default\" name=\"fallback\"]]def_val[[/template]]|",
            "[[quid id=\"out\"/]]|",
            "[[cors src=\"stackjsiframe.min.js\"/]]|",
            "[[jsstring]]he\"llo[[/jsstring]]|",
            "[[entityescape]]<b>\"x\"</b>[[/entityescape]]"
        );
        let mut counter = 0;
        let rendered = render_castext(
            &make_ft(raw),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            MatrixParens::Square,
            &mut counter,
        );
        assert_eq!(
            rendered,
            "custom();|def_val|stack-quid-out|/corsscripts/stackjsiframe.min.js|\"he\\\"llo\"|&lt;b&gt;&quot;x&quot;&lt;/b&gt;"
        );
    }

    #[test]
    fn test_stack_js_javascript_block_and_imports() {
        let raw = "[[javascript input-ref-ans1=\"ans1Ref\"]]import {foo} from 'cors://bar.js'; ans1Ref.value = '1';[[/javascript]]";
        let mut counter = 0;
        let rendered = render_castext(
            &make_ft(raw),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            MatrixParens::Square,
            &mut counter,
        );
        assert!(rendered.contains("class=\"stack-iframe-holder\""));
        assert!(rendered.contains("display:none;"));
        assert!(rendered.contains("&lt;script type=&quot;module&quot;&gt;"));
        assert!(rendered.contains("import stack_js from &#39;/corsscripts/stackjsiframe.min.js&#39;;"));
        assert!(rendered.contains("import {foo} from &#39;/corsscripts/bar.js&#39;;"));
        assert!(rendered.contains(
            "Promise.all([stack_js.request_access_to_input(&quot;ans1&quot;,true)]).then(([ans1Ref]) =&gt; {"
        ));
    }

    #[test]
    fn test_stack_js_stray_script_and_style_errors() {
        let raw = "[[script]]alert(1);[[/script]] and [[style]]body{color:red;}[[/style]]";
        let mut counter = 0;
        let rendered = render_castext(
            &make_ft(raw),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            MatrixParens::Square,
            &mut counter,
        );
        assert!(rendered.contains(" ERROR [[script]] blocks must be within iframes. "));
        assert!(rendered.contains(" ERROR [[style]] blocks must be within iframes. "));
        assert!(!rendered.contains("<script"));
        assert!(!rendered.contains("<style"));
    }

    #[test]
    fn test_stack_js_jsxgraph_and_iframe_blocks() {
        let raw = concat!(
            "[[jsxgraph width=\"400px\" height=\"300px\" input-ref-ans1=\"ans1Ref\"]]",
            "var board = JXG.JSXGraph.initBoard(divid, {});",
            "[[/jsxgraph]]",
            "[[iframe width=\"200px\" height=\"100px\" title=\"Custom Frame\"]]",
            "[[style]]body { margin: 0; }[[/style]]",
            "[[script type=\"module\"]]console.log(FRAME_ID);[[/script]]",
            "<div id=\"inner\">Hello</div>",
            "[[/iframe]]"
        );
        let mut counter = 0;
        let rendered = render_castext(
            &make_ft(raw),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            MatrixParens::Square,
            &mut counter,
        );
        assert!(rendered.contains("class=\"jxgbox stack-jsxgraph stack-iframe-holder\""));
        assert!(rendered.contains("data-jxg-code="));
        assert!(rendered.contains("<iframe id=\"stack-iframe-"));
        assert!(rendered.contains("sandbox=\"allow-scripts allow-downloads\""));
        assert!(!rendered.contains("allow-same-origin"));
        assert!(rendered.contains("/corsscripts/jsxgraphcore.min.js"));
        assert!(rendered.contains("/corsscripts/stackjsiframe.min.js"));
        assert!(rendered.contains("/corsscripts/stackjsxgraph.min.js"));
        assert!(rendered.contains(
            "Promise.all([stack_js.request_access_to_input(&quot;ans1&quot;,true)]).then(([ans1Ref]) =&gt; {"
        ));
        assert!(rendered.contains("data-frame-title=\"Custom Frame\""));
        assert!(rendered.contains("&lt;style&gt;body { margin: 0; }&lt;/style&gt;"));
        assert!(rendered.contains("&lt;script type=&quot;module&quot;&gt;console.log(FRAME_ID);&lt;/script&gt;"));
    }

    #[test]
    fn test_stack_js_sample_questions() {
        let samples = [
            include_str!(
                "../../../moodle-qtype_stack/samplequestions/stack-js/stack-js-issue-specific-example-938.xml"
            ),
            include_str!(
                "../../../moodle-qtype_stack/samplequestions/stackdemo/40-JSX_graph_interactive.xml"
            ),
            include_str!(
                "../../../moodle-qtype_stack/samplequestions/stacklibrary/Doc-Examples/Specialist-Tools-Docs/STACK-JS/Validation-state-listener.xml"
            ),
        ];

        for xml in samples {
            let quiz = parse_moodle_xml(xml).expect("should parse sample STACK-JS question XML");
            let exported = export_quiz_to_moodle_xml(&quiz).expect("should export sample XML");
            let reparsed = parse_moodle_xml(&exported).expect("should reparse exported XML");
            assert_eq!(quiz, reparsed, "export roundtrip must be lossless");
            let sq = quiz
                .question
                .iter()
                .find_map(|q| match q {
                    Question::Stack(s) => Some(s),
                    _ => None,
                })
                .expect("should contain a StackQuestion");
            let ev = render_stack_question_static(sq, 12345);
            assert!(
                ev.questiontext_html.contains("stack-iframe-holder"),
                "rendered questiontext_html should contain stack-iframe-holder"
            );
            assert!(
                ev.questiontext_html.contains("<iframe"),
                "rendered questiontext_html should contain iframe element"
            );
            assert!(
                ev.questiontext_html.contains("sandbox=\"allow-scripts allow-downloads\""),
                "iframe element must have sandbox without allow-same-origin"
            );
            assert!(
                !ev.questiontext_html.contains("allow-same-origin"),
                "iframe element must never grant allow-same-origin"
            );
            assert!(
                ev.questiontext_html.contains("data-stack-input-type="),
                "rendered questiontext_html should contain data-stack-input-type"
            );
        }
    }
}
