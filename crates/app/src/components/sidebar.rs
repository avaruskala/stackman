use leptos::prelude::*;
use web_sys::HtmlSelectElement;

use crate::{
    models::{QuestionEntry, QuestionListView, QuestionTypeFilter},
    state::{AppState, AppStateExt, AppStateStoreFields},
};

#[component]
pub fn QuestionSidebar() -> impl IntoView {
    let state = AppState::use_context();
    let list_view = state.list_view();
    let search_query = state.search_query();
    let type_filter = state.type_filter();
    let category_filter = state.category_filter();
    let is_dragging = state.is_dragging();

    view! {
        <aside
            class="question-sidebar"
            class:drag-active=move || is_dragging.get()
            on:dragover=move |ev| state.handle_drag_over(ev)
            on:dragleave=move |ev| state.handle_drag_leave(ev)
            on:drop=move |ev| state.handle_drop(ev)
        >
            <Show when=move || { state.total_count() > 0 }>
                <div class="sidebar-controls">
                    <div class="sidebar-view-tabs" role="tablist">
                        <button
                            type="button"
                            class="view-tab-btn"
                            class:active=move || list_view.get() == QuestionListView::All
                            on:click=move |_| list_view.set(QuestionListView::All)
                        >
                            "Kaikki kysymykset, "
                            {move || state.total_count()}
                        </button>
                        <button
                            type="button"
                            class="view-tab-btn"
                            class:active=move || list_view.get() == QuestionListView::Chosen
                            on:click=move |_| list_view.set(QuestionListView::Chosen)
                        >
                            "Valitut, "
                            {move || state.chosen_count()}
                        </button>
                    </div>

                    <div class="search-box-wrapper">
                        <input
                            type="search"
                            class="search-input"
                            placeholder="Hae kysymyksen nimellä, kategorialla tai tägillä..."
                            prop:value=move || search_query.get()
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="sidebar-row">
                        <select
                            class="filter-select"
                            title="Kysymystyyppi"
                            on:change=move |ev| {
                                let target: HtmlSelectElement = event_target(&ev);
                                match target.selected_index() {
                                    0 => type_filter.set(QuestionTypeFilter::All),
                                    1 => type_filter.set(QuestionTypeFilter::Stack),
                                    2 => type_filter.set(QuestionTypeFilter::Multichoice),
                                    3 => type_filter.set(QuestionTypeFilter::TrueFalse),
                                    _ => {}
                                }
                            }
                        >
                            <option>"Kaikki, " {move || state.total_count()}</option>
                            <option>"Stack, " {move || state.stack_count()}</option>
                            <option>"Monivalinta, " {move || state.mc_count()}</option>
                            <option>"Tosi tai epätosi, " {move || state.tf_count()}</option>
                        </select>

                        <Show when=move || { state.available_categories().len() > 1 }>
                            <select
                                class="filter-select"
                                title="Kategoria"
                                prop:value=move || category_filter.get()
                                on:change=move |ev| category_filter.set(event_target_value(&ev))
                            >
                                <option value="all">"Kaikki kategoriat"</option>
                                {move || {
                                    state
                                        .available_categories()
                                        .into_iter()
                                        .map(|cat| {
                                            let c_val = cat.clone();
                                            view! { <option value=c_val>{cat}</option> }
                                        })
                                        .collect_view()
                                }}
                            </select>
                        </Show>
                    </div>

                    <div class="sidebar-row sidebar-selection-actions">
                        <button
                            type="button"
                            class="btn btn-sm btn-secondary"
                            on:click=move |_| state.select_filtered()
                        >
                            "Valitse suodatetut"
                        </button>

                        <Show when=move || { state.chosen_count() > 0 }>
                            <button
                                type="button"
                                class="btn btn-sm btn-secondary"
                                on:click=move |_| state.clear_chosen()
                            >
                                "Poista valinnat"
                            </button>
                        </Show>
                    </div>
                </div>
            </Show>

            <div class="question-list-scroll">
                <Show
                    when=move || state.has_questions()
                    fallback=move || {
                        view! {
                            <div class="empty-sidebar-dropzone">
                                <h3>"Ei ladattuja kysymyksiä"</h3>
                                <p>
                                    "Pudota Moodle XML -tiedosto tähän tai tuo tiedosto yläpalkista."
                                </p>
                                <label class="btn btn-primary">
                                    "Valitse XML-tiedostoja"
                                    <input
                                        type="file"
                                        multiple
                                        accept=".xml,text/xml,application/xml"
                                        class="hidden-file-input"
                                        on:change=move |ev| state.handle_file_change(ev)
                                    />
                                </label>
                            </div>
                        }
                    }
                >
                    <Show
                        when=move || !state.filtered_questions().is_empty()
                        fallback=move || {
                            view! {
                                <div class="empty-sidebar-dropzone">
                                    {move || {
                                        if list_view.get() == QuestionListView::Chosen && state.chosen_count() == 0 {
                                            view! {
                                                <div>
                                                    <h3>"Ei valittuja kysymyksiä"</h3>
                                                    <p>
                                                        "Merkitse kysymyksiä valintaruudulla tai esikatselun yläpalkista viedäksesi ne XML-tiedostoon."
                                                    </p>
                                                    <button
                                                        type="button"
                                                        class="btn btn-secondary"
                                                        on:click=move |_| list_view.set(QuestionListView::All)
                                                    >
                                                        "Näytä kaikki kysymykset"
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div>
                                                    <h3>"Ei suodattimia vastaavia kysymyksiä"</h3>
                                                    <p>"Muuta hakusanaa tai suodattimia."</p>
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            }
                        }
                    >
                        <ul class="question-items-list">
                            <For
                                each=move || state.filtered_questions()
                                key=|item| item.id
                                children=move |item| {
                                    view! { <QuestionListItem item=item /> }
                                }
                            />
                        </ul>
                    </Show>
                </Show>
            </div>
        </aside>
    }
    .into_any()
}

#[component]
fn QuestionListItem(item: QuestionEntry) -> impl IntoView {
    let state = AppState::use_context();
    let selected_id = state.selected_id();
    let id = item.id;
    let badge_class = item.badge_class();
    let badge_label = item.badge_short_label();
    let tags = item.tags.clone();
    let is_chosen = move || state.is_chosen(id);

    view! {
        <li
            class="question-list-item"
            class=("selected", move || selected_id.get() == Some(id))
            class=("chosen-item", is_chosen)
            on:click=move |_| selected_id.set(Some(id))
        >
            <div class="q-item-top">
                <label
                    class="q-item-check-label"
                    title="Valitse kysymys vientilistalle"
                    on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
                >
                    <input
                        type="checkbox"
                        class="q-item-checkbox"
                        prop:checked=is_chosen
                        on:change=move |_| state.toggle_chosen(id)
                    />
                </label>
                <span class=badge_class>{badge_label}</span>
                <span class="q-item-category" title=item.category.clone()>
                    {item.category.clone()}
                </span>
                <span class="q-item-grade">{format!("{:.1}p", item.defaultgrade)}</span>
            </div>
            <div class="q-item-title">{item.name.clone()}</div>
            <div class="q-item-meta">
                <span class="q-item-source" title=item.source_file>
                    {item.source_file.clone()}
                </span>
                <div class="q-item-tags">
                    {tags
                        .into_iter()
                        .take(3)
                        .map(|t| view! { <span class="tag-chip">{t}</span> })
                        .collect_view()}
                </div>
            </div>
        </li>
    }
}
