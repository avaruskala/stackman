use std::collections::BTreeSet;

use leptos::prelude::*;
use reactive_stores::Store;
use serde::{Deserialize, Serialize};
use stack::{models::Quiz, render::export_quiz_to_moodle_xml};

use crate::{
    api::download_xml_file,
    models::{
        LanguageFilter, QuestionEntry, QuestionListView, QuestionTypeFilter, build_export_quiz,
        extract_questions_from_quiz,
    },
};

pub const LOCAL_STORAGE_KEY: &str = "stackman_app_state";

#[cfg(target_arch = "wasm32")]
fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

#[cfg(not(target_arch = "wasm32"))]
fn get_local_storage() -> Option<web_sys::Storage> {
    None
}

#[cfg(all(target_arch = "wasm32", feature = "hydrate"))]
pub fn setup_beforeunload_save(store: Store<AppState>) {
    use wasm_bindgen::{JsCast, closure::Closure};
    if let Some(win) = web_sys::window() {
        let closure = Closure::<dyn Fn()>::new(move || {
            store.with_untracked(|s| s.save_to_local_storage());
        });
        let _ =
            win.add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref());
        closure.forget();
    }
}

#[cfg(not(all(target_arch = "wasm32", feature = "hydrate")))]
pub fn setup_beforeunload_save(_store: Store<AppState>) {}

/// Application state managed as a fine-grained reactive store via `reactive_stores`.
#[derive(Debug, Clone, PartialEq, Store, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    #[store(key: usize = |entry| entry.id)]
    pub questions: Vec<QuestionEntry>,
    pub selected_id: Option<usize>,
    pub chosen_ids: BTreeSet<usize>,
    pub list_view: QuestionListView,
    #[serde(skip)]
    pub parse_error: Option<String>,
    #[serde(skip)]
    pub is_dragging: bool,
    #[serde(skip)]
    pub loading_msg: Option<String>,

    // Sidebar filter fields
    pub search_query: String,
    pub type_filter: QuestionTypeFilter,
    pub category_filter: String,

    // Preview toolbar fields
    pub lang_filter: LanguageFilter,
    pub seed: u32,
    pub fill_correct: bool,
    pub show_general_feedback: bool,
    pub show_variables: bool,
    pub show_prt: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            questions: Vec::new(),
            selected_id: None,
            chosen_ids: BTreeSet::new(),
            list_view: QuestionListView::All,
            parse_error: None,
            is_dragging: false,
            loading_msg: None,
            search_query: String::new(),
            type_filter: QuestionTypeFilter::All,
            category_filter: String::from("all"),
            lang_filter: LanguageFilter::All,
            seed: 12345,
            fill_correct: false,
            show_general_feedback: true,
            show_variables: true,
            show_prt: false,
        }
    }
}

impl AppState {
    /// Serialize the state to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize state from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Attempts to load the saved `AppState` from browser `localStorage`.
    /// Returns `None` if running outside the browser, if `localStorage` is inaccessible,
    /// if the key does not exist, or if deserialization fails.
    pub fn load_from_local_storage() -> Option<Self> {
        let storage = get_local_storage()?;
        match storage.get_item(LOCAL_STORAGE_KEY) {
            Ok(Some(json)) if !json.trim().is_empty() => match Self::from_json(&json) {
                Ok(state) => {
                    log::info!(
                        "Loaded AppState from localStorage ({} questions)",
                        state.questions.len()
                    );
                    Some(state)
                }
                Err(err) => {
                    log::warn!("Failed to deserialize AppState from localStorage: {err}");
                    None
                }
            },
            Ok(_) => None,
            Err(err) => {
                log::warn!("Error accessing localStorage: {err:?}");
                None
            }
        }
    }

    /// Saves the current `AppState` to browser `localStorage`.
    pub fn save_to_local_storage(&self) {
        if let Some(storage) = get_local_storage() {
            if self.questions.is_empty() && self.chosen_ids.is_empty() && self.selected_id.is_none()
            {
                let _ = storage.remove_item(LOCAL_STORAGE_KEY);
                log::debug!("Cleared empty AppState from localStorage");
                return;
            }
            match self.to_json() {
                Ok(json) => {
                    if let Err(err) = storage.set_item(LOCAL_STORAGE_KEY, &json) {
                        log::warn!("Failed to write AppState to localStorage: {err:?}");
                    } else {
                        log::debug!("Saved AppState to localStorage");
                    }
                }
                Err(err) => {
                    log::warn!("Failed to serialize AppState: {err}");
                }
            }
        }
    }

    /// Clears the saved `AppState` from browser `localStorage`.
    pub fn clear_local_storage() {
        if let Some(storage) = get_local_storage()
            && let Err(err) = storage.remove_item(LOCAL_STORAGE_KEY)
        {
            log::warn!("Failed to remove AppState from localStorage: {err:?}");
        }
    }

    /// Initialize `Store<AppState>` and provide it to the Leptos context.
    pub fn provide() -> Store<Self> {
        Self::provide_ext(false)
    }

    /// Initialize `Store<AppState>` with optional immediate loading from `localStorage`.
    pub fn provide_ext(load_immediate: bool) -> Store<Self> {
        let initial = if load_immediate {
            Self::load_from_local_storage().unwrap_or_default()
        } else {
            Self::default()
        };
        let store = Store::new(initial);
        provide_context(store);
        setup_beforeunload_save(store);
        store
    }

    /// Retrieve `Store<AppState>` from the Leptos context.
    pub fn use_context() -> Store<Self> {
        expect_context::<Store<Self>>()
    }
}

/// Extension methods on `Store<AppState>` for reactive queries and state mutations.
pub trait AppStateExt {
    fn filtered_questions(&self) -> Vec<QuestionEntry>;
    fn available_categories(&self) -> Vec<String>;
    fn selected_question(&self) -> Option<QuestionEntry>;
    fn total_count(&self) -> usize;
    fn chosen_count(&self) -> usize;
    fn stack_count(&self) -> usize;
    fn mc_count(&self) -> usize;
    fn tf_count(&self) -> usize;
    fn has_questions(&self) -> bool;
    fn is_chosen(&self, id: usize) -> bool;

    fn add_parsed_quiz(&self, filename: String, quiz: Quiz);
    fn toggle_chosen(&self, id: usize);
    fn select_filtered(&self);
    fn clear_chosen(&self);
    fn export_chosen(&self);
    fn clear_all(&self);
    fn randomize_seed(&self);
    fn handle_file_change(&self, ev: leptos::ev::Event);
    fn handle_drag_over(&self, ev: leptos::ev::DragEvent);
    fn handle_drag_leave(&self, ev: leptos::ev::DragEvent);
    fn handle_drop(&self, ev: leptos::ev::DragEvent);

    fn save_to_local_storage(&self);
    fn load_from_local_storage(&self) -> bool;
    fn clear_local_storage(&self);
}

impl AppStateExt for Store<AppState> {
    fn filtered_questions(&self) -> Vec<QuestionEntry> {
        let q_str = self.search_query().get().to_lowercase();
        let t_filter = self.type_filter().get();
        let c_filter = self.category_filter().get();
        let v_mode = self.list_view().get();

        self.chosen_ids().with(|chosen| {
            self.questions().with(|all_questions| {
                all_questions
                    .iter()
                    .filter(|item| {
                        let view_ok = match v_mode {
                            QuestionListView::All => true,
                            QuestionListView::Chosen => chosen.contains(&item.id),
                        };
                        let type_ok = t_filter.matches(&item.qtype);
                        let cat_ok = c_filter == "all" || item.category == c_filter;
                        let search_ok = item.matches_search(&q_str);
                        view_ok && type_ok && cat_ok && search_ok
                    })
                    .cloned()
                    .collect()
            })
        })
    }

    fn available_categories(&self) -> Vec<String> {
        self.questions().with(|all_questions| {
            let mut cats = BTreeSet::new();
            for q in all_questions {
                cats.insert(q.category.clone());
            }
            cats.into_iter().collect()
        })
    }

    fn selected_question(&self) -> Option<QuestionEntry> {
        let sel = self.selected_id().get()?;
        self.questions()
            .with(|all_questions| all_questions.iter().find(|q| q.id == sel).cloned())
    }

    fn total_count(&self) -> usize {
        self.questions().with(|q| q.len())
    }

    fn chosen_count(&self) -> usize {
        self.chosen_ids().with(|c| c.len())
    }

    fn stack_count(&self) -> usize {
        self.questions()
            .with(|q| q.iter().filter(|item| item.qtype == "stack").count())
    }

    fn mc_count(&self) -> usize {
        self.questions()
            .with(|q| q.iter().filter(|item| item.qtype == "multichoice").count())
    }

    fn tf_count(&self) -> usize {
        self.questions()
            .with(|q| q.iter().filter(|item| item.qtype == "truefalse").count())
    }

    fn has_questions(&self) -> bool {
        self.questions().with(|q| !q.is_empty())
    }

    fn is_chosen(&self, id: usize) -> bool {
        self.chosen_ids().with(|set| set.contains(&id))
    }

    fn add_parsed_quiz(&self, filename: String, quiz: Quiz) {
        let mut first_new_id = None;
        self.questions().update(|current| {
            let next_id = current.last().map(|q| q.id + 1).unwrap_or(1);
            let new_entries = extract_questions_from_quiz(quiz, &filename, next_id);
            first_new_id = new_entries.first().map(|e| e.id);
            current.extend(new_entries);
        });
        if self.selected_id().get_untracked().is_none()
            && let Some(id) = first_new_id
        {
            self.selected_id().set(Some(id));
        }
    }

    fn toggle_chosen(&self, id: usize) {
        self.chosen_ids().update(|set| {
            if !set.remove(&id) {
                set.insert(id);
            }
        });
    }

    fn select_filtered(&self) {
        let ids: Vec<usize> = self
            .filtered_questions()
            .into_iter()
            .map(|q| q.id)
            .collect();
        self.chosen_ids().update(|set| {
            for id in ids {
                set.insert(id);
            }
        });
    }

    fn clear_chosen(&self) {
        self.chosen_ids().set(BTreeSet::new());
    }

    fn export_chosen(&self) {
        let entries: Vec<QuestionEntry> = self.chosen_ids().with(|chosen| {
            self.questions().with(|all| {
                all.iter()
                    .filter(|q| chosen.contains(&q.id))
                    .cloned()
                    .collect()
            })
        });
        if entries.is_empty() {
            return;
        }
        let quiz = build_export_quiz(&entries);
        match export_quiz_to_moodle_xml(&quiz) {
            Ok(xml) => {
                download_xml_file("valitut-kysymykset.xml", &xml);
            }
            Err(err) => {
                self.parse_error().set(Some(err));
            }
        }
    }

    fn clear_all(&self) {
        self.questions().set(Vec::new());
        self.selected_id().set(None);
        self.chosen_ids().set(BTreeSet::new());
        self.list_view().set(QuestionListView::All);
        self.parse_error().set(None);
        AppState::clear_local_storage();
    }

    fn randomize_seed(&self) {
        let next = self
            .seed()
            .get_untracked()
            .wrapping_mul(1664525)
            .wrapping_add(1013904223)
            % 999999
            + 1;
        self.seed().set(next);
    }

    fn handle_file_change(&self, _ev: leptos::ev::Event) {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            let state = *self;
            if let Some(target) = _ev.target()
                && let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>()
                && let Some(files) = input.files()
            {
                crate::api::read_uploaded_xml_files(files, state, move |filename, quiz| {
                    state.add_parsed_quiz(filename, quiz)
                });
                input.set_value("");
            }
        }
    }

    fn handle_drag_over(&self, ev: leptos::ev::DragEvent) {
        ev.prevent_default();
        self.is_dragging().set(true);
    }

    fn handle_drag_leave(&self, ev: leptos::ev::DragEvent) {
        ev.prevent_default();
        self.is_dragging().set(false);
    }

    fn handle_drop(&self, ev: leptos::ev::DragEvent) {
        ev.prevent_default();
        self.is_dragging().set(false);
        #[cfg(feature = "hydrate")]
        {
            let state = *self;
            if let Some(dt) = ev.data_transfer()
                && let Some(files) = dt.files()
            {
                crate::api::read_uploaded_xml_files(files, state, move |filename, quiz| {
                    state.add_parsed_quiz(filename, quiz)
                });
            }
        }
    }

    fn save_to_local_storage(&self) {
        self.with_untracked(|state| state.save_to_local_storage());
    }

    fn load_from_local_storage(&self) -> bool {
        if let Some(loaded) = AppState::load_from_local_storage() {
            self.update(|s| *s = loaded);
            true
        } else {
            false
        }
    }

    fn clear_local_storage(&self) {
        AppState::clear_local_storage();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack::render::parse_moodle_xml;
    const RAW_XML: &str = include_str!(
        "../../../material/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723/questions-Discrete-Multilang.-20161122-0033.xml"
    );

    #[test]
    fn test_reactive_store_state_transitions() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::provide();
            let ctx_state = AppState::use_context();

            assert_eq!(ctx_state.total_count(), 0);
            assert!(!ctx_state.has_questions());

            let quiz = parse_moodle_xml(RAW_XML).expect("should parse XML");
            state.add_parsed_quiz("discrete.xml".to_string(), quiz);

            let total = state.total_count();
            assert!(total > 0);
            assert!(state.has_questions());
            assert_eq!(state.selected_id().get_untracked(), Some(1));
            assert_eq!(state.filtered_questions().len(), total);

            // Toggle question 1 and 2 into chosen list
            state.toggle_chosen(1);
            state.toggle_chosen(2);
            assert_eq!(state.chosen_count(), 2);
            assert!(state.is_chosen(1));
            assert!(state.is_chosen(2));

            // Switch view to Chosen only via store field lens
            state.list_view().set(QuestionListView::Chosen);
            assert_eq!(state.filtered_questions().len(), 2);

            // Unchoose question 1
            state.toggle_chosen(1);
            assert_eq!(state.chosen_count(), 1);
            assert_eq!(state.filtered_questions().len(), 1);

            // Switch back to All and clear
            state.list_view().set(QuestionListView::All);
            assert_eq!(state.filtered_questions().len(), total);

            state.clear_all();
            assert_eq!(state.total_count(), 0);
            assert_eq!(state.chosen_count(), 0);
            assert_eq!(state.selected_id().get_untracked(), None);
        });
    }

    #[test]
    fn test_app_state_json_roundtrip() {
        let quiz = parse_moodle_xml(RAW_XML).expect("should parse XML");
        let entries = extract_questions_from_quiz(quiz, "discrete.xml", 1);
        assert!(!entries.is_empty());

        let state = AppState {
            questions: entries,
            selected_id: Some(1),
            chosen_ids: BTreeSet::from([1, 2, 3]),
            list_view: QuestionListView::Chosen,
            search_query: "induktio".to_string(),
            type_filter: QuestionTypeFilter::Stack,
            category_filter: "Matematiikka".to_string(),
            lang_filter: LanguageFilter::Fi,
            seed: 98765,
            fill_correct: true,
            show_general_feedback: false,
            show_variables: false,
            show_prt: true,
            ..Default::default()
        };

        // Serialize to JSON
        let json = state.to_json().expect("should serialize AppState to JSON");
        assert!(!json.is_empty());

        // Deserialize from JSON
        let deserialized = match AppState::from_json(&json) {
            Ok(s) => s,
            Err(e) => {
                let col = e.column();
                let start = col.saturating_sub(100);
                let end = (col + 100).min(json.len());
                panic!("Error: {e}\nContext: {}", &json[start..end]);
            }
        };

        assert_eq!(deserialized.questions.len(), state.questions.len());
        assert_eq!(deserialized.selected_id, Some(1));
        assert_eq!(deserialized.chosen_ids, state.chosen_ids);
        assert_eq!(deserialized.list_view, QuestionListView::Chosen);
        assert_eq!(deserialized.search_query, "induktio");
        assert_eq!(deserialized.type_filter, QuestionTypeFilter::Stack);
        assert_eq!(deserialized.category_filter, "Matematiikka");
        assert_eq!(deserialized.lang_filter, LanguageFilter::Fi);
        assert_eq!(deserialized.seed, 98765);
        assert!(deserialized.fill_correct);
        assert!(!deserialized.show_general_feedback);
        assert!(!deserialized.show_variables);
        assert!(deserialized.show_prt);

        // Skipped transient fields should be default
        assert_eq!(deserialized.parse_error, None);
        assert!(!deserialized.is_dragging);
        assert_eq!(deserialized.loading_msg, None);
    }

    #[test]
    fn test_app_state_json_defaults() {
        let partial_json = r#"{"search_query": "test"}"#;
        let deserialized: AppState =
            AppState::from_json(partial_json).expect("should deserialize partial JSON");
        assert_eq!(deserialized.search_query, "test");
        assert_eq!(deserialized.seed, 12345);
        assert_eq!(deserialized.list_view, QuestionListView::All);
        assert_eq!(deserialized.questions.len(), 0);
        assert_eq!(deserialized.chosen_ids.len(), 0);
    }

    #[test]
    fn test_on_cleanup_execution() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let cleaned_up = Arc::new(AtomicBool::new(false));
        let cleaned_up_clone = Arc::clone(&cleaned_up);

        let owner = Owner::new();
        owner.with(|| {
            let _state = AppState::provide();
            on_cleanup(move || {
                cleaned_up_clone.store(true, Ordering::SeqCst);
            });
        });

        assert!(!cleaned_up.load(Ordering::SeqCst));
        drop(owner);
        assert!(cleaned_up.load(Ordering::SeqCst));
    }

    #[test]
    fn test_load_from_storage_method() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::provide();
            // In native test environment without browser localStorage, returns false gracefully
            assert!(!state.load_from_local_storage());
        });
    }
}
