pub mod multichoice;
pub mod stack;
pub mod truefalse;

use ::stack::{models::Question, render::EvaluatedStackQuestion};
use leptos::{either::EitherOf4, prelude::*};

use crate::{
    components::toolbar::PreviewToolbar,
    models::QuestionEntry,
    state::{AppState, AppStateExt, AppStateStoreFields},
};

use self::{
    multichoice::MultichoiceQuestionPreview, stack::StackQuestionPreview,
    truefalse::TrueFalseQuestionPreview,
};

/// Right-hand preview pane that displays the selected question.
#[component]
pub fn QuestionPreviewPane(
    stack_eval_resource: Resource<Option<EvaluatedStackQuestion>>,
) -> impl IntoView {
    let state = AppState::use_context();
    let lang_filter = state.lang_filter();

    view! {
        <section class="preview-main-panel">
            <Show when=move || {
                state.selected_question().is_some()
            }>
                {move || {
                    let entry = state.selected_question()?;
                    let lang_cls = lang_filter.get().container_class();
                    Some(
                        view! {
                            <div class="preview-container">
                                <PreviewToolbar entry=entry.clone() />

                                <div class=lang_cls>
                                    {match entry.question.clone() {
                                        Question::Stack(sq) => {
                                            EitherOf4::A(
                                                view! {
                                                    <StackQuestionPreview
                                                        entry=entry.clone()
                                                        question=*sq
                                                        eval_resource=stack_eval_resource
                                                    />
                                                },
                                            )
                                        }
                                        Question::Multichoice(mq) => {
                                            EitherOf4::B(
                                                view! {
                                                    <MultichoiceQuestionPreview
                                                        entry=entry.clone()
                                                        question=mq
                                                    />
                                                },
                                            )
                                        }
                                        Question::TrueFalse(tf) => {
                                            EitherOf4::C(
                                                view! {
                                                    <TrueFalseQuestionPreview
                                                        entry=entry.clone()
                                                        question=tf
                                                    />
                                                },
                                            )
                                        }
                                        Question::Category(_) => EitherOf4::D(()),
                                    }}
                                </div>
                            </div>
                        },
                    )
                }}
            </Show>
        </section>
    }
    .into_any()
}

/// Shared left-hand Moodle question metadata box (`moodle-info-box`).
#[component]
pub fn MoodleInfoBox(
    entry: QuestionEntry,
    #[prop(optional)] subtitle: Option<&'static str>,
    #[prop(optional)] extra: Option<AnyView>,
) -> impl IntoView {
    let badge_class = entry.badge_class();
    let badge_label = entry.badge_full_label();
    let tags = entry.tags.clone();

    view! {
        <div class="moodle-info-box">
            <div class="info-qnum">"Kysymys " {entry.id}</div>
            <div class="info-qtype">
                <span class=badge_class>{badge_label}</span>
            </div>
            <div class="info-grade">
                "Oletuspisteet " <strong>{format!("{:.2}", entry.defaultgrade)}</strong>
            </div>
            {subtitle.map(|sub| view! { <div class="info-penalty">{sub}</div> })}

            {extra}

            {(!tags.is_empty())
                .then_some({
                    view! {
                        <div class="info-tags">
                            {tags
                                .into_iter()
                                .map(|t| view! { <span class="tag-chip">{t}</span> })
                                .collect_view()}
                        </div>
                    }
                })}
        </div>
    }
    .into_any()
}
