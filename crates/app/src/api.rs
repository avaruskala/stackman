use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use reactive_stores::{Store};
#[cfg(feature = "hydrate")]
use stack::{models::Quiz, render::parse_moodle_xml};
use stack::{models::stack::StackQuestion, render::EvaluatedStackQuestion};

#[cfg(feature = "hydrate")]
use crate::state::{AppState, AppStateStoreFields};

#[server]
pub async fn evaluate_stack_preview(
    question: StackQuestion,
    seed: u32,
) -> Result<EvaluatedStackQuestion, ServerFnError> {
    let evaluated = tokio::task::spawn_blocking(move || {
        stack::render::evaluate_stack_question_maxima(&question, seed)
    })
    .await
    .map_err(|e| ServerFnError::new(format!("Maxima task join error: {e}")))?;
    Ok(evaluated)
}

#[cfg(feature = "hydrate")]
pub fn trigger_client_render(fill_correct: bool) {
    use wasm_bindgen::JsValue;
    if let Some(win) = web_sys::window()
        && let Ok(fn_val) = js_sys::Reflect::get(&win, &JsValue::from_str("stackmanRenderPreview"))
        && let Ok(func) = wasm_bindgen::JsCast::dyn_into::<js_sys::Function>(fn_val)
    {
        let _ = func.call1(&JsValue::NULL, &JsValue::from_bool(fill_correct));
    }
}

#[cfg(not(feature = "hydrate"))]
pub fn trigger_client_render(_fill_correct: bool) {}

#[cfg(feature = "hydrate")]
pub fn download_xml_file(filename: &str, xml_content: &str) {
    use wasm_bindgen::{JsCast, JsValue};
    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(xml_content));
    let bag = web_sys::BlobPropertyBag::new();
    bag.set_type("application/xml;charset=utf-8");

    if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &bag)
        && let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob)
        && let Some(win) = web_sys::window()
        && let Some(doc) = win.document()
        && let Ok(el) = doc.create_element("a")
        && let Ok(anchor) = el.dyn_into::<web_sys::HtmlAnchorElement>()
    {
        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.click();
        let _ = web_sys::Url::revoke_object_url(&url);
    }
}

#[cfg(not(feature = "hydrate"))]
pub fn download_xml_file(_filename: &str, _xml_content: &str) {}

#[cfg(feature = "hydrate")]
pub fn read_uploaded_xml_files(
    file_list: web_sys::FileList,
    state: Store<AppState>,
    on_quiz_parsed: impl Fn(String, Quiz) + 'static,
) {
    let count = file_list.length();
    if count == 0 {
        return;
    }
    state
        .loading_msg()
        .set(Some(format!("Luetaan {count} XML-tiedostoa...")));
    state.parse_error().set(None);

    leptos::task::spawn_local(async move {
        let mut errors = Vec::new();
        let mut loaded_any = false;

        leptos::logging::log!("loading start");
        for i in 0..count {
            if let Some(file) = file_list.get(i) {
                let filename = file.name();
                match wasm_bindgen_futures::JsFuture::from(file.text()).await {
                    Ok(js_text) => {
                        let xml_content = js_text.as_string().unwrap_or_default();
                        match parse_moodle_xml(&xml_content) {
                            Ok(quiz) => {
                                on_quiz_parsed(filename, quiz);
                                loaded_any = true;
                            }
                            Err(err) => {
                                errors.push(format!("{filename}: {err}"));
                            }
                        }
                    }
                    Err(err) => {
                        errors.push(format!("{filename}: tiedoston luku epäonnistui ({err:?})"));
                    }
                }
            }
        }

        leptos::logging::log!("loading done");

        state.loading_msg().set(None);
        if !errors.is_empty() {
            state.parse_error().set(Some(errors.join(" | ")));
        } else if loaded_any {
            state.parse_error().set(None);
        }
    });
}
