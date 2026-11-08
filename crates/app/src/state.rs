use std::collections::BTreeSet;

use leptos::prelude::*;
use reactive_stores::Store;
use stack::{models::Quiz, render::export_quiz_to_moodle_xml};

use crate::{
    api::download_xml_file,
    models::{
        LanguageFilter, QuestionEntry, QuestionListView, QuestionTypeFilter, build_export_quiz,
        extract_questions_from_quiz,
    },
};

/// Application state managed as a fine-grained reactive store via `reactive_stores`.
#[derive(Debug, Clone, Store)]
pub struct AppState {
    #[store(key: usize = |entry| entry.id)]
    pub questions: Vec<QuestionEntry>,
    pub selected_id: Option<usize>,
    pub chosen_ids: BTreeSet<usize>,
    pub list_view: QuestionListView,
    pub parse_error: Option<String>,
    pub is_dragging: bool,
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
    /// Initialize `Store<AppState>` and provide it to the Leptos context.
    pub fn provide() -> Store<Self> {
        let store = Store::new(Self::default());
        provide_context(store);
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
                crate::api::read_uploaded_xml_files(
                    files,
                    state,
                    move |filename, quiz| state.add_parsed_quiz(filename, quiz),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack::render::parse_moodle_xml;

    #[test]
    fn test_reactive_store_state_transitions() {
        let owner = Owner::new();
        owner.with(|| {
            let state = AppState::provide();
            let ctx_state = AppState::use_context();

            assert_eq!(ctx_state.total_count(), 0);
            assert!(!ctx_state.has_questions());

            let raw_xml = include_str!(
                "../../../material/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723/questions-Discrete-Multilang.-20161122-0033.xml"
            );
            let quiz = parse_moodle_xml(raw_xml).expect("should parse XML");
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
}
