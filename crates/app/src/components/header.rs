use leptos::prelude::*;

use crate::{
    models::QuestionListView,
    state::{AppState, AppStateExt, AppStateStoreFields},
};

#[component]
pub fn AppHeader() -> impl IntoView {
    let state = AppState::use_context();
    let list_view = state.list_view();

    view! {
        <header class="app-header">
            <div class="brand-section">
                <div class="brand-logo">"S"</div>
                <div>
                    <h1 class="brand-title">"Stackman"</h1>
                    <p class="brand-subtitle">"Moodle- ja STACK-kysymysten esikatselu"</p>
                </div>
            </div>

            <div class="header-actions">
                <Show when=move || { state.chosen_count() > 0 }>
                    <button
                        type="button"
                        class="btn btn-secondary"
                        on:click=move |_| {
                            if list_view.get() == QuestionListView::Chosen {
                                list_view.set(QuestionListView::All);
                            } else {
                                list_view.set(QuestionListView::Chosen);
                            }
                        }
                    >
                        {move || {
                            if list_view.get() == QuestionListView::Chosen {
                                format!("Näytä kaikki, {}", state.total_count())
                            } else {
                                format!("Näytä valitut, {}", state.chosen_count())
                            }
                        }}
                    </button>

                    <button
                        type="button"
                        class="btn btn-export"
                        on:click=move |_| state.export_chosen()
                    >
                        "Vie valitut XML, "
                        {move || state.chosen_count()}
                        " kpl"
                    </button>
                </Show>

                <Show when=move || { state.total_count() > 0 }>
                    <button
                        type="button"
                        class="btn btn-ghost-danger"
                        on:click=move |_| state.clear_all()
                    >
                        "Tyhjennä lista, "
                        {move || state.total_count()}
                        " kpl"
                    </button>
                </Show>

                <label
                    class="btn btn-primary upload-btn-label"
                    on:dragover=move |ev| state.handle_drag_over(ev)
                    on:dragleave=move |ev| state.handle_drag_leave(ev)
                    on:drop=move |ev| state.handle_drop(ev)
                >
                    "Tuo XML-tiedostoja"
                    <input
                        type="file"
                        multiple
                        accept=".xml,text/xml,application/xml"
                        class="hidden-file-input"
                        on:change=move |ev| state.handle_file_change(ev)
                    />
                </label>
            </div>
        </header>
    }
}

#[component]
pub fn StatusBanners() -> impl IntoView {
    let state = AppState::use_context();
    let loading_msg = state.loading_msg();
    let parse_error = state.parse_error();

    view! {
        <Show when=move || loading_msg.get().is_some()>
            <div class="status-banner info-banner">
                <span class="spinner-dot"></span>
                <span>{move || loading_msg.get().unwrap_or_default()}</span>
            </div>
        </Show>
        <Show when=move || parse_error.get().is_some()>
            <div class="status-banner error-banner">
                <span>"Jäsennysvirhe, " {move || parse_error.get().unwrap_or_default()}</span>
                <button type="button" class="banner-close" on:click=move |_| parse_error.set(None)>
                    "Sulje"
                </button>
            </div>
        </Show>
    }
}
