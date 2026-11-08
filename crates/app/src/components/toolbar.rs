use leptos::prelude::*;

use crate::{
    models::{LanguageFilter, QuestionEntry},
    state::{AppState, AppStateExt, AppStateStoreFields},
};

#[component]
pub fn PreviewToolbar(entry: QuestionEntry) -> impl IntoView {
    let state = AppState::use_context();
    let lang_filter = state.lang_filter();
    let seed = state.seed();
    let fill_correct = state.fill_correct();
    let show_general_feedback = state.show_general_feedback();
    let show_variables = state.show_variables();
    let show_prt = state.show_prt();

    let is_stack = entry.is_stack();
    let entry_id = entry.id;
    let is_chosen = move || state.is_chosen(entry_id);

    view! {
        <div class="preview-toolbar">
            <div class="toolbar-title-group">
                <div class="toolbar-breadcrumbs">
                    <span class="crumb-file">{entry.source_file.clone()}</span>
                    <span class="crumb-sep">"›"</span>
                    <span class="crumb-cat">{entry.category.clone()}</span>
                </div>
                <h2 class="toolbar-question-title">{entry.name.clone()}</h2>
            </div>

            <div class="toolbar-controls-row">
                <button
                    type="button"
                    class="btn btn-sm btn-choose-toggle"
                    class:active=is_chosen
                    on:click=move |_| state.toggle_chosen(entry_id)
                >
                    {move || {
                        if is_chosen() {
                            "Valittu vientilistalle"
                        } else {
                            "Valitse vientilistalle"
                        }
                    }}
                </button>

                <div class="segmented-control" title="Monikielisten osioiden suodatus">
                    <button
                        type="button"
                        class="seg-btn"
                        class:active=move || lang_filter.get() == LanguageFilter::All
                        on:click=move |_| lang_filter.set(LanguageFilter::All)
                    >
                        "Kaikki"
                    </button>
                    <button
                        type="button"
                        class="seg-btn"
                        class:active=move || lang_filter.get() == LanguageFilter::Fi
                        on:click=move |_| lang_filter.set(LanguageFilter::Fi)
                    >
                        "FI"
                    </button>
                    <button
                        type="button"
                        class="seg-btn"
                        class:active=move || lang_filter.get() == LanguageFilter::En
                        on:click=move |_| lang_filter.set(LanguageFilter::En)
                    >
                        "EN"
                    </button>
                </div>

                <Show when=move || is_stack>
                    <div class="seed-control-group">
                        <label class="control-label">
                            "Siemen"
                            <input
                                type="number"
                                class="seed-number-input"
                                prop:value=move || seed.get().to_string()
                                on:change=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                        seed.set(v);
                                    }
                                }
                            />
                        </label>
                        <button
                            type="button"
                            class="btn btn-sm btn-secondary"
                            title="Arvo uusi satunnainen kysymysversio"
                            on:click=move |_| state.randomize_seed()
                        >
                            "Uusi versio"
                        </button>
                    </div>
                </Show>

                <div class="toggle-buttons-group">
                    <button
                        type="button"
                        class="btn btn-sm btn-toggle"
                        class:active=move || fill_correct.get()
                        on:click=move |_| fill_correct.update(|v| *v = !*v)
                    >
                        "Täytä oikeat vastaukset"
                    </button>
                    <button
                        type="button"
                        class="btn btn-sm btn-toggle"
                        class:active=move || show_general_feedback.get()
                        on:click=move |_| show_general_feedback.update(|v| *v = !*v)
                    >
                        "Malliratkaisu ja palaute"
                    </button>
                    <Show when=move || is_stack>
                        <button
                            type="button"
                            class="btn btn-sm btn-toggle"
                            class:active=move || show_variables.get()
                            on:click=move |_| show_variables.update(|v| *v = !*v)
                        >
                            "Muuttujat"
                        </button>
                        <button
                            type="button"
                            class="btn btn-sm btn-toggle"
                            class:active=move || show_prt.get()
                            on:click=move |_| show_prt.update(|v| *v = !*v)
                        >
                            "Vastauspuu"
                        </button>
                    </Show>
                </div>
            </div>
        </div>
    }
}
