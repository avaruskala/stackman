#![recursion_limit = "256"]

pub mod api;
pub mod components;
pub mod models;
pub mod state;

use crate::api::{evaluate_stack_preview, trigger_client_render};
use crate::components::{
    header::{AppHeader, StatusBanners},
    preview::QuestionPreviewPane,
    sidebar::QuestionSidebar,
};
use crate::state::{AppState, AppStateExt, AppStateStoreFields};
use ::stack::{models::Question, render::render_stack_question_static};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

/// Client-side MathJax + JSXGraph + STACK input initialization script injected into the HTML `<head>`.
const CLIENT_INIT_SCRIPT: &str = include_str!("client-init-script.js");

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="fi">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <script>{CLIENT_INIT_SCRIPT}</script>
                <script
                    id="MathJax-script"
                    async
                    src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js"
                ></script>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/stackman.css" />
        <Title text="Stackman, Moodle- ja STACK-kysymysten esikatselu" />

        <Router>
            <Routes fallback=|| "Sivua ei löytynyt.".into_view()>
                <Route path=StaticSegment("") view=HomePage />
            </Routes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let state = AppState::provide();
    let seed = state.seed();

    // Evaluate STACK question on the server whenever selected STACK question or seed changes
    let stack_eval_resource = Resource::new(
        move || {
            let sq_opt = state
                .selected_question()
                .and_then(|entry| match entry.question {
                    Question::Stack(sq) => Some((entry.id, *sq)),
                    _ => None,
                });
            (sq_opt, seed.get())
        },
        move |(sq_opt, current_seed)| async move {
            let (_id, sq) = sq_opt?;
            match evaluate_stack_preview(sq.clone(), current_seed).await {
                Ok(ev) => Some(ev),
                Err(err) => {
                    let mut fallback = render_stack_question_static(&sq, current_seed);
                    fallback.maxima_error = Some(err.to_string());
                    Some(fallback)
                }
            }
        },
    );

    // Trigger client-side MathJax, JSXGraph, and fill-correct whenever preview updates
    Effect::new(move |_| {
        state.selected_id().track();
        stack_eval_resource.track();
        state.lang_filter().track();
        state.show_general_feedback().track();
        state.show_variables().track();
        state.show_prt().track();
        let fill = state.fill_correct().get();
        trigger_client_render(fill);
    });

    view! {
        <div class="app-layout">
            <AppHeader />
            <StatusBanners />

            <div class="workspace-split">
                <QuestionSidebar />
                <QuestionPreviewPane stack_eval_resource=stack_eval_resource />
            </div>
        </div>
    }
}
