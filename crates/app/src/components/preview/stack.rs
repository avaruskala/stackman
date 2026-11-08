use ::stack::{
    models::stack::StackQuestion,
    render::{
        EvaluatedInput, EvaluatedPrt, EvaluatedStackQuestion, EvaluatedVariable,
        render_stack_question_static,
    },
};
use leptos::either::Either;
use leptos::prelude::*;
use reactive_stores::Field;

use crate::{
    components::preview::MoodleInfoBox,
    models::QuestionEntry,
    state::{AppState, AppStateStoreFields},
};

/// Renders a STACK question with Maxima CAS evaluation (or static fallback),
/// model answers, general feedback, question variables, and Potential Response Trees (PRTs).
#[component]
pub fn StackQuestionPreview(
    entry: QuestionEntry,
    question: StackQuestion,
    eval_resource: Resource<Option<EvaluatedStackQuestion>>,
) -> impl IntoView {
    let state = AppState::use_context();
    let seed = state.seed();
    let fill_correct: Field<bool> = state.fill_correct().into();
    let show_general_feedback = state.show_general_feedback();
    let show_variables = state.show_variables();
    let show_prt = state.show_prt();

    let deployed_seeds = question.deployedseed.clone();
    let q_vars_raw = question.questionvariables.text.clone();
    let fallback_sq = question.clone();

    view! {
        <Suspense fallback=move || {
            view! {
                <div class="moodle-que-card loading-card">
                    <div class="spinner-dot"></div>
                    <span>"Evaluoidaan STACK-kysymystä Maximalla..."</span>
                </div>
            }
        }>
            {move || {
                let ev = eval_resource
                    .get()
                    .flatten()
                    .unwrap_or_else(|| render_stack_question_static(&fallback_sq, seed.get()));
                let deployed = deployed_seeds.clone();
                let raw_vars = q_vars_raw.clone();
                let penalty_text = format!("Vähennys {:.0} %", entry.penalty * 100.0);
                let extra_info = view! {
                    <div class="info-penalty">
                        <span>{penalty_text}</span>
                    </div>
                    <div class="info-cas-status">
                        {if ev.maxima_ok {
                            Either::Left(
                                view! { <span class="cas-badge cas-ok">"Maxima CAS"</span> },
                            )
                        } else {
                            Either::Right(
                                view! {
                                    <span
                                        class="cas-badge cas-fallback"
                                        title=ev.maxima_error.clone().unwrap_or_default()
                                    >
                                        "Staattinen esikatselu"
                                    </span>
                                },
                            )
                        }}
                    </div>

                    {if !deployed.is_empty() {
                        Either::Left(
                            view! {
                                <div class="info-deployed-seeds">
                                    <label class="deployed-label">
                                        "Käyttöönotetut versiot"
                                    </label>
                                    <select
                                        class="deployed-select"
                                        on:change=move |e| {
                                            if let Ok(s) = event_target_value(&e).parse::<u32>() {
                                                seed.set(s);
                                            }
                                        }
                                    >
                                        {deployed
                                            .iter()
                                            .enumerate()
                                            .map(|(i, &s)| {
                                                view! {
                                                    <option
                                                        value=s.to_string()
                                                        selected=move || seed.get() == s
                                                    >
                                                        {format!("Versio {}, siemen {}", i + 1, s)}
                                                    </option>
                                                }
                                            })
                                            .collect_view()}
                                    </select>
                                </div>
                            },
                        )
                    } else {
                        Either::Right(())
                    }}
                }
                    .into_any();

                view! {
                    <div class="moodle-que-layout">
                        <MoodleInfoBox entry=entry.clone() extra=extra_info />

                        <div class="moodle-main-column">
                            <StackFormulationCard
                                seed_value=ev.seed
                                questiontext_html=ev.questiontext_html.clone()
                                inputs=ev.inputs.clone()
                                fill_correct=fill_correct
                            />

                            <Show when=move || show_general_feedback.get()>
                                <StackGeneralFeedbackCard
                                    generalfeedback_html=ev.generalfeedback_html.clone()
                                    questionnote_html=ev.questionnote_html.clone()
                                />
                            </Show>

                            <Show when=move || show_variables.get()>
                                <StackVariablesCard
                                    seed_value=ev.seed
                                    raw_variables=raw_vars.clone()
                                    variables=ev.variables.clone()
                                />
                            </Show>

                            <Show when=move || show_prt.get()>
                                <StackPrtCard prts=ev.prts.clone() />
                            </Show>
                        </div>
                    </div>
                }
                    .into_any()
            }}
        </Suspense>
    }
}

/// Renders the STACK question text, interactive inputs, and toggleable teacher model answers.
#[component]
fn StackFormulationCard(
    seed_value: u32,
    questiontext_html: String,
    inputs: Vec<EvaluatedInput>,
    fill_correct: Field<bool>,
) -> impl IntoView {
    let has_inputs = !inputs.is_empty();

    view! {
        <div class="moodle-formulation-card formulation" class:show-validation-hints=move || fill_correct.get()>
            <div class="formulation-header">
                <span class="formulation-label">"Kysymysteksti"</span>
                <span class="seed-pill">{format!("Siemen {seed_value}")}</span>
            </div>
            <div class="moodle-questiontext tex2jax_process" inner_html=questiontext_html></div>

            <Show when=move || fill_correct.get() && has_inputs>
                <div class="model-answers-box">
                    <h4>"Oikeat mallivastaukset"</h4>
                    <div class="model-answers-grid">
                        {inputs
                            .iter()
                            .map(|inp| {
                                let type_str = format!("{:?}", inp.input_type);
                                let latex_html = if inp.tans_latex.is_empty() {
                                    inp.tans_value.clone()
                                } else {
                                    format!("\\({}\\)", inp.tans_latex)
                                };
                                view! {
                                    <div class="model-answer-item">
                                        <div class="ma-header">
                                            <code class="ma-name">{inp.name.clone()}</code>
                                            <span class="ma-type">{type_str}</span>
                                            <span class="ma-expr">
                                                "Mallivastaus " <code>{inp.tans_expr.clone()}</code>
                                            </span>
                                        </div>
                                        <div
                                            class="ma-rendered tex2jax_process"
                                            inner_html=latex_html
                                        ></div>
                                        <div class="ma-raw">
                                            <span>"Maxima-syöte "</span>
                                            <code>{inp.tans_value.clone()}</code>
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </Show>
        </div>
    }
    .into_any()
}

/// Renders the STACK worked solution (`generalfeedback`) and `questionnote`.
#[component]
fn StackGeneralFeedbackCard(
    generalfeedback_html: String,
    questionnote_html: String,
) -> impl IntoView {
    let has_general_feedback = !generalfeedback_html.trim().is_empty();
    let has_questionnote = !questionnote_html.trim().is_empty();

    view! {
        <div class="moodle-feedback-card outcome">
            <div class="card-section-header">
                <h3>"Malliratkaisu ja yleinen palaute"</h3>
            </div>
            {if has_general_feedback {
                Either::Left(
                    view! {
                        <div
                            class="moodle-generalfeedback tex2jax_process"
                            inner_html=generalfeedback_html
                        ></div>
                    },
                )
            } else {
                Either::Right(
                    view! {
                        <p class="muted-text">
                            "Kysymykselle ei ole määritelty yleistä palautetta."
                        </p>
                    },
                )
            }}

            {if has_questionnote {
                Either::Right(
                    view! {
                        <div class="question-note-box">
                            <strong>"Kysymysmuistiinpano "</strong>
                            <span class="tex2jax_process" inner_html=questionnote_html></span>
                        </div>
                    },
                )
            } else {
                Either::Left(())
            }}
        </div>
    }
}

/// Renders the raw Maxima `questionvariables` code alongside the table of evaluated CAS values.
#[component]
fn StackVariablesCard(
    seed_value: u32,
    raw_variables: String,
    variables: Vec<EvaluatedVariable>,
) -> impl IntoView {
    view! {
        <div class="moodle-variables-card">
            <div class="card-section-header">
                <h3>"Kysymysmuuttujat ja Maxima-arvot"</h3>
            </div>
            <div class="variables-split">
                <div class="variables-code-pane">
                    <h4>"Maxima-koodi"</h4>
                    {if raw_variables.trim().is_empty() {
                        Either::Left(view! { <p class="muted-text">"Ei kysymysmuuttujia."</p> })
                    } else {
                        Either::Right(
                            view! {
                                <pre class="maxima-code-block">
                                    <code>{raw_variables}</code>
                                </pre>
                            },
                        )
                    }}
                </div>
                <div class="variables-table-pane">
                    <h4>"Evaluoidut arvot siemenellä " {seed_value}</h4>
                    {if variables.is_empty() {
                        Either::Left(
                            view! { <p class="muted-text">"Ei evaluoituja muuttujia."</p> }
                                .into_any(),
                        )
                    } else {
                        Either::Right(
                            view! {
                                <div class="vars-table-scroll">
                                    <table class="vars-table">
                                        <thead>
                                            <tr>
                                                <th>"Muuttuja"</th>
                                                <th>"LaTeX-muoto"</th>
                                                <th>"Maxima-arvo"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {variables
                                                .into_iter()
                                                .map(|v| {
                                                    let tex_str = format!("\\({}\\)", v.latex);
                                                    view! {
                                                        <tr>
                                                            <td class="var-name-cell">
                                                                <code>{v.name}</code>
                                                            </td>
                                                            <td
                                                                class="var-latex-cell tex2jax_process"
                                                                inner_html=tex_str
                                                            ></td>
                                                            <td class="var-raw-cell">
                                                                <code>{v.value}</code>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            },
                        )
                    }}
                </div>
            </div>
        </div>
    }
}

/// Renders the Potential Response Trees (PRTs) and their grading nodes/branches.
#[component]
fn StackPrtCard(prts: Vec<EvaluatedPrt>) -> impl IntoView {
    view! {
        <div class="moodle-prt-card outcome">
            <div class="card-section-header">
                <h3>"Vastauspuut"</h3>
            </div>
            {prts
                .into_iter()
                .map(|prt| {
                    let meta_str = format!(
                        "Arvo {:.2} p, automaattinen sievennys {}",
                        prt.value,
                        if prt.autosimplify { "päällä" } else { "pois" },
                    );
                    view! {
                        <div class="prt-block">
                            <div class="prt-header">
                                <span class="prt-title">"Vastauspuu " <code>{prt.name}</code></span>
                                <span class="prt-meta">{meta_str}</span>
                            </div>
                            {if !prt.feedbackvariables.trim().is_empty() {
                                Either::Right(
                                    view! {
                                        <div class="prt-fbvars">
                                            <strong>"Palautemuuttujat"</strong>
                                            <pre class="maxima-code-block">
                                                <code>{prt.feedbackvariables}</code>
                                            </pre>
                                        </div>
                                    },
                                )
                            } else {
                                Either::Left(())
                            }}
                            <div class="prt-nodes-list">
                                {prt
                                    .nodes
                                    .into_iter()
                                    .map(|node| {
                                        let true_next = if node.truenextnode == -1 {
                                            "lopetus".to_string()
                                        } else {
                                            format!("solmuun {}", node.truenextnode)
                                        };
                                        let false_next = if node.falsenextnode == -1 {
                                            "lopetus".to_string()
                                        } else {
                                            format!("solmuun {}", node.falsenextnode)
                                        };
                                        let opt_suffix = if !node.testoptions.is_empty() {
                                            format!(", valinnat {}", node.testoptions)
                                        } else {
                                            String::new()
                                        };

                                        view! {
                                            <div class="prt-node-card">
                                                <div class="prt-node-top">
                                                    <span class="node-badge">"Solmu " {node.name}</span>
                                                    <span class="node-test-pill">{node.answertest}</span>
                                                    <span class="node-args">
                                                        "Opiskelijan vastaus " <code>{node.sans}</code>
                                                        ", mallivastaus " <code>{node.tans}</code> {opt_suffix}
                                                    </span>
                                                </div>
                                                <div class="prt-branches-grid">
                                                    <div class="prt-branch branch-true">
                                                        <div class="branch-header">
                                                            <strong>"Kun ehto täyttyy"</strong>
                                                            <span>
                                                                {format!(
                                                                    "Pisteet {} {:.2}, jatko {}",
                                                                    node.truescoremode,
                                                                    node.truescore,
                                                                    true_next,
                                                                )}
                                                            </span>
                                                            <code class="ansnote-chip">{node.trueanswernote}</code>
                                                        </div>
                                                        {if !node.truefeedback_html.trim().is_empty() {
                                                            Either::Left(
                                                                view! {
                                                                    <div
                                                                        class="branch-feedback tex2jax_process"
                                                                        inner_html=node.truefeedback_html
                                                                    ></div>
                                                                },
                                                            )
                                                        } else {
                                                            Either::Right(())
                                                        }}
                                                    </div>
                                                    <div class="prt-branch branch-false">
                                                        <div class="branch-header">
                                                            <strong>"Kun ehto ei täyty"</strong>
                                                            <span>
                                                                {format!(
                                                                    "Pisteet {} {:.2}, jatko {}",
                                                                    node.falsescoremode,
                                                                    node.falsescore,
                                                                    false_next,
                                                                )}
                                                            </span>
                                                            <code class="ansnote-chip">{node.falseanswernote}</code>
                                                        </div>
                                                        {if !node.falsefeedback_html.trim().is_empty() {
                                                            Either::Left(
                                                                view! {
                                                                    <div
                                                                        class="branch-feedback tex2jax_process"
                                                                        inner_html=node.falsefeedback_html
                                                                    ></div>
                                                                }
                                                                    .into_any(),
                                                            )
                                                        } else {
                                                            Either::Right(())
                                                        }}
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}
