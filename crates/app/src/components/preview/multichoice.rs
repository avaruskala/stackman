use std::collections::BTreeSet;

use ::stack::{
    models::multichoice::MultichoiceQuestion,
    render::{format_answer_number, resolve_pluginfiles},
};
use leptos::prelude::*;

use crate::{
    components::preview::MoodleInfoBox,
    models::QuestionEntry,
    state::{AppState, AppStateStoreFields},
};

/// Renders a Moodle `multichoice` question with single/multiple selection and answer feedback.
#[component]
pub fn MultichoiceQuestionPreview(
    entry: QuestionEntry,
    question: MultichoiceQuestion,
) -> impl IntoView {
    let state = AppState::use_context();
    let fill_correct = state.fill_correct();
    let show_general_feedback = state.show_general_feedback();

    let qtext_html = resolve_pluginfiles(&question.questiontext);
    let gen_fb_html = resolve_pluginfiles(&question.generalfeedback);
    let has_gen_fb = !gen_fb_html.trim().is_empty();
    let single = question.single;
    let numbering = question.answernumbering;
    let answers = question.answer.clone();
    let selected_indices = RwSignal::new(BTreeSet::<usize>::new());
    let q_id = entry.id;

    let subtitle = if single {
        "Yksi valinta"
    } else {
        "Useita valintoja"
    };

    view! {
        <div class="moodle-que-layout">
            <MoodleInfoBox entry=entry subtitle=subtitle />

            <div class="moodle-main-column">
                <div class="moodle-formulation-card">
                    <div class="formulation-header">
                        <span class="formulation-label">"Kysymysteksti"</span>
                    </div>
                    <div class="moodle-questiontext tex2jax_process" inner_html=qtext_html></div>

                    <div class="mc-Interaction-area">
                        <p class="mc-prompt">
                            {if single {
                                "Valitse yksi vaihtoehto."
                            } else {
                                "Valitse yksi tai useampi vaihtoehto."
                            }}
                        </p>
                        <div class="mc-answers-list">
                            {answers
                                .into_iter()
                                .enumerate()
                                .map(|(idx, ans)| {
                                    let num_prefix = format_answer_number(numbering, idx);
                                    let is_positive = ans.fraction > 0.0;
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
                                            let checked = selected_indices.get().contains(&idx)
                                                || (fill_correct.get() && is_positive);
                                            if fill_correct.get() && is_positive {
                                                "mc-answer-row answer-correct-highlight"
                                            } else if checked {
                                                "mc-answer-row answer-selected"
                                            } else {
                                                "mc-answer-row"
                                            }
                                        }>
                                            <label class="mc-answer-label">
                                                <input
                                                    type=if single { "radio" } else { "checkbox" }
                                                    name=format!("mc_q_{q_id}")
                                                    prop:checked=move || {
                                                        selected_indices.get().contains(&idx)
                                                            || (fill_correct.get() && is_positive)
                                                    }
                                                    on:change=move |_| {
                                                        selected_indices
                                                            .update(|set| {
                                                                if single {
                                                                    set.clear();
                                                                    set.insert(idx);
                                                                } else if !set.remove(&idx) {
                                                                    set.insert(idx);
                                                                }
                                                            });
                                                    }
                                                />
                                                <span class="mc-ans-number">{num_prefix}</span>
                                                <span
                                                    class="mc-ans-text tex2jax_process"
                                                    inner_html=ans_text
                                                ></span>
                                            </label>
                                            <Show when=move || {
                                                fill_correct.get() || selected_indices.get().contains(&idx)
                                            }>
                                                <div class="mc-ans-meta">
                                                    <span class=frac_cls>{format!("{fraction:.0}%")}</span>
                                                    {(!ans_fb.trim().is_empty())
                                                        .then_some(
                                                            view! {
                                                                <div
                                                                    class="mc-ans-feedback tex2jax_process"
                                                                    inner_html=ans_fb.clone()
                                                                ></div>
                                                            },
                                                        )}
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
