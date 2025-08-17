use crate::admin::overview::admin_captions::AdminCaptionsPage;
use crate::admin::overview::admin_monitor::AdminMonitorsPage;
use crate::admin::overview::admin_queue::AdminQueuePage;
use crate::admin::overview::admin_videos::AdminVideosPage;
use crate::admin::overview::AdminPage;
use crate::models::SearchResult;
use crate::search::api::execute_search;
use crate::search::components::{ResultsList, SearchBar};
use crate::search::utils::{get_filter_param, get_query_param};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/admin")]
    Admin,
    #[at("/admin/videos")]
    AdminVideos,
    #[at("/admin/captions")]
    AdminCaptions,
    #[at("/admin/monitors")]
    AdminMonitors,
    #[at("/admin/queue")]
    AdminQueue,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <SearchApp /> },
        Route::Admin => html! { <AdminPage /> },
        Route::AdminVideos => html! { <AdminVideosPage /> },
        Route::AdminCaptions => html! { <AdminCaptionsPage /> },
        Route::AdminMonitors => html! { <AdminMonitorsPage /> },
        Route::AdminQueue => html! { <AdminQueuePage /> },
        Route::NotFound => html! {
            <div class="min-h-screen flex items-center justify-center bg-gray-700">
                <div class="bg-white p-8 rounded-lg shadow-lg text-center">
                    <h1 class="text-2xl font-bold text-gray-800 mb-4">{"404 - Page Not Found"}</h1>
                    <Link<Route> to={Route::Home} classes="text-blue-600 hover:underline">
                        {"Go back to search"}
                    </Link<Route>>
                </div>
            </div>
        },
    }
}

#[function_component(SearchApp)]
pub fn search_app() -> Html {
    let search_query = use_state(|| get_query_param().unwrap_or_default());
    let search_results = use_state(Vec::<SearchResult>::default);
    let total_results = use_state(|| None::<(usize, usize)>); // Add total results state
    let loading = use_state(|| false);
    let error_message = use_state(Option::<String>::default);
    let init_done = use_state(|| false);
    let current_page = use_state(|| 0usize);

    let filter_param = get_filter_param();
    let is_wide_search = use_state(|| filter_param.unwrap().search_type == "wide");

    let on_wide_search_toggle = {
        let is_wide_search = is_wide_search.clone();
        let current_page = current_page.clone();
        Callback::from(move |_| {
            is_wide_search.set(!*is_wide_search);
            current_page.set(0);
        })
    };

    // Effect for initial load
    {
        let search_query = search_query.clone();
        let search_results = search_results.clone();
        let total_results = total_results.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let init_done = init_done.clone();
        let is_wide_search = is_wide_search.clone();
        let current_page = current_page.clone();

        use_effect(move || {
            if !*init_done {
                if let Some(query) = get_query_param() {
                    search_query.set(query.clone());
                    loading.set(true);
                    error_message.set(None);

                    let is_wide = *is_wide_search;
                    let search_type = if is_wide { "wide" } else { "natural" };
                    let page = *current_page;
                    wasm_bindgen_futures::spawn_local(async move {
                        execute_search(
                            query,
                            search_type,
                            page,
                            search_results,
                            total_results,
                            error_message,
                            loading,
                        )
                        .await;
                    });
                }
                init_done.set(true);
            }
            || ()
        });
    }

    // Callback for search execution
    let on_search = {
        let search_query = search_query.clone();
        let search_results = search_results.clone();
        let total_results = total_results.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let is_wide_search = is_wide_search.clone();
        let current_page = current_page.clone();

        Callback::from(move |query: String| {
            let search_results = search_results.clone();
            let total_results = total_results.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            let current_page = current_page.clone();

            search_query.set(query.clone());
            current_page.set(0);
            loading.set(true);
            error_message.set(None);

            let is_wide = *is_wide_search;
            let search_type = if is_wide { "wide" } else { "natural" };
            wasm_bindgen_futures::spawn_local(async move {
                execute_search(
                    query,
                    search_type,
                    0,
                    search_results,
                    total_results,
                    error_message,
                    loading,
                )
                .await;
            });
        })
    };

    // Callback for page changes
    let on_page_change = {
        let search_query = search_query.clone();
        let search_results = search_results.clone();
        let total_results = total_results.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let is_wide_search = is_wide_search.clone();
        let current_page = current_page.clone();

        Callback::from(move |page: usize| {
            let search_query = search_query.clone();
            let search_results = search_results.clone();
            let total_results = total_results.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            let current_page = current_page.clone();

            current_page.set(page);
            loading.set(true);
            error_message.set(None);

            let query = (*search_query).clone();
            let is_wide = *is_wide_search;
            let search_type = if is_wide { "wide" } else { "natural" };
            wasm_bindgen_futures::spawn_local(async move {
                execute_search(
                    query,
                    search_type,
                    page,
                    search_results,
                    total_results,
                    error_message,
                    loading,
                )
                .await;
            });
        })
    };

    html! {
        <div class="min-h-screen flex flex-col items-center justify-center bg-gray-700 p-4">
            <div class="bg-white p-8 rounded-lg shadow-lg w-full max-w-2xl">
                <h1 class="text-3xl font-bold text-center text-gray-800 mb-6">
                    {"YouTube Caption Search"}
                </h1>

                <div class="text-center mb-4">
                    <Link<Route> to={Route::Admin} classes="text-blue-600 hover:underline text-sm">
                        {"Admin Panel"}
                    </Link<Route>>
                </div>

                <SearchBar
                    query={(*search_query).clone()}
                    loading={*loading}
                    on_search={on_search}
                />

                <div class="flex items-center justify-center mb-4">
                    <label class="inline-flex items-center">
                        <input
                            type="checkbox"
                            class="form-checkbox h-5 w-5 text-blue-600"
                            checked={*is_wide_search}
                            onchange={on_wide_search_toggle}
                        />
                        <span class="ml-2 text-gray-700">{"Enable wide search"}</span>
                    </label>
                </div>

                {
                    if let Some(msg) = &*error_message {
                        html! {
                            <p class="text-red-600 text-center mb-4">{ format!("Error: {msg}") }</p>
                        }
                    } else {
                        html! {}
                    }
                }

                <ResultsList
                    results={(*search_results).clone()}
                    loading={*loading}
                    error={(*error_message).clone()}
                    query={(*search_query).clone()}
                    on_page_change={on_page_change}
                    current_page={*current_page}
                    total_results={*total_results}
                />
            </div>
        </div>
    }
}
