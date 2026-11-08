use ::stack::{models::truefalse::TrueFalseQuestion, render::resolve_pluginfiles};
use leptos::prelude::*;

use crate::{
    components::preview::MoodleInfoBox,
    models::QuestionEntry,
    state::{AppState, AppStateStoreFields},
};

/// Renders a Moodle `truefalse` question with interactive radio buttons and feedback.
#[component]
pub fn TrueFalseQuestionPreview(
    entry: QuestionEntry,
    question: TrueFalseQuestion,
) -> impl IntoView {
    let state = AppState::use_context();
    let fill_correct = state.fill_correct();
    let show_general_feedback = state.show_general_feedback();

    let qtext_html = resolve_pluginfiles(&question.questiontext);
    let gen_fb_html = resolve_pluginfiles(&question.generalfeedback);
    let has_gen_fb = !gen_fb_html.trim().is_empty();
    let answers = question.answer.clone();
    let selected_idx = RwSignal::new(Option::<usize>::None);
    let q_id = entry.id;

    view! {
        <div class="moodle-que-layout">
            <MoodleInfoBox entry=entry />

            <div class="moodle-main-column">
                <div class="moodle-formulation-card">
                    <div class="formulation-header">
                        <span class="formulation-label">"Kysymysteksti"</span>
                    </div>
                    <div class="moodle-questiontext tex2jax_process" inner_html=qtext_html></div>

                    <div class="mc-Interaction-area">
                        <p class="mc-prompt">"Valitse yksi vaihtoehto."</p>
                        <div class="mc-answers-list">
                            {answers
                                .into_iter()
                                .enumerate()
                                .map(|(idx, ans)| {
                                    let is_correct = ans.fraction > 0.0;
                                    let ans_fb = ans.feedback.text.clone();
                                    let ans_text = ans.text.clone();
                                    let fraction = ans.fraction;
                                    let frac_cls = if fraction > 0.0 {
                                        "fraction-badge frac-pos"
                                    } else {
                                        "fraction-badge frac-zero"
                                    };

                                    view! {
                                        <div class=move || {
                                            if fill_correct.get() && is_correct {
                                                "mc-answer-row answer-correct-highlight"
                                            } else if selected_idx.get() == Some(idx) {
                                                "mc-answer-row answer-selected"
                                            } else {
                                                "mc-answer-row"
                                            }
                                        }>
                                            <label class="mc-answer-label">
                                                <input
                                                    type="radio"
                                                    name=format!("tf_q_{q_id}")
                                                    prop:checked=move || {
                                                        selected_idx.get() == Some(idx)
                                                            || (fill_correct.get() && is_correct)
                                                    }
                                                    on:change=move |_| selected_idx.set(Some(idx))
                                                />
                                                <span class="mc-ans-text">{ans_text}</span>
                                            </label>
                                            <Show when=move || {
                                                fill_correct.get() || selected_idx.get() == Some(idx)
                                            }>
                                                <div class="mc-ans-meta">
                                                    <span class=frac_cls>
                                                        {if fraction > 0.0 { "Oikein" } else { "Väärin" }}
                                                    </span>
                                                    {(!ans_fb.trim().is_empty())
                                                        .then_some({
                                                            view! {
                                                                <div
                                                                    class="mc-ans-feedback tex2jax_process"
                                                                    inner_html=ans_fb.clone()
                                                                ></div>
                                                            }
                                                        })}
                                                </div>
                                            </Show>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                </div>

                <Show when=move || show_general_feedback.get() && has_gen_fb>
                    <div class="moodle-feedback-card">
                        <div class="card-section-header">
                            <h3>"Yleinen palaute"</h3>
                        </div>
                        <div
                            class="moodle-generalfeedback tex2jax_process"
                            inner_html=gen_fb_html.clone()
                        ></div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
