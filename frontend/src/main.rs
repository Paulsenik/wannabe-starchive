mod models;

use crate::models::{SearchResult, VideoMetadata};
use gloo_net::http::Request;
use web_sys::MouseEvent;
use web_sys::{wasm_bindgen, HtmlInputElement};
use yew::prelude::*;

// Main App component
fn format_timestamp(seconds: f64) -> String {
    let minutes = (seconds as u32) / 60;
    let remaining_seconds = (seconds as u32) % 60;
    format!("{:02}:{:02}", minutes, remaining_seconds)
}

#[function_component(SearchBar)]
fn search_bar(props: &SearchBarProps) -> Html {
    html! {
        <div class="flex flex-col sm:flex-row gap-4 mb-6">
            <input
                type="text"
                class="flex-grow p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Enter search query..."
                value={props.query.clone()}
                oninput={props.on_input.clone()}
                onkeydown={props.on_enter.clone()}
            />
            <button
                class="px-6 py-3 bg-blue-600 text-white font-semibold rounded-md shadow-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition duration-200 ease-in-out"
                onclick={props.on_search.clone()}
                disabled={props.loading}
            >
                { if props.loading { "Searching..." } else { "Search" } }
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SearchBarProps {
    pub query: String,
    pub loading: bool,
    pub on_input: Callback<InputEvent>,
    pub on_search: Callback<MouseEvent>,
    pub on_enter: Callback<KeyboardEvent>,
}

#[function_component(SearchResultItem)]
fn search_result_item(props: &SearchResultItemProps) -> Html {
    html! {
        <div class="p-4 bg-white">
            <p class="text-sm text-gray-500 mb-1">
                <a href={format!("https://www.youtube.com/watch?v={}&t={}s", props.result.video_id, props.result.start_time)}
                   target="_blank"
                   class="ml-2 text-blue-600 hover:underline">
                {format!("{} ↗ ", format_timestamp(props.result.start_time))}
                </a>
                {
                    if let Some(highlight) = &props.result.highlighted_text {
                        Html::from_html_unchecked(AttrValue::from(highlight.clone()))
                    } else {
                        html! { &props.result.text }
                    }
                }
            </p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SearchResultItemProps {
    pub result: SearchResult,
}

#[derive(Properties, PartialEq)]
pub struct VideoResultsProps {
    pub video_id: String,
    pub results: Vec<SearchResult>,
    pub metadata: Option<VideoMetadata>,
}

#[function_component(VideoResults)]
fn video_results(props: &VideoResultsProps) -> Html {
    let expanded = use_state(|| false);
    let video_metadata = use_state(|| None);
    let error_message = use_state(|| None);
    let loading = use_state(|| false);

    {
        let video_id = props.video_id.clone();
        let video_metadata = video_metadata.clone();
        let error_message = error_message.clone();
        let loading = loading.clone();
        let prev_video_id = use_state(|| String::new());

        use_effect(move || {
            if *prev_video_id != video_id {
                prev_video_id.set(video_id.clone());
                loading.set(true);
                error_message.set(None);

                wasm_bindgen_futures::spawn_local(async move {
                    get_video_metadata(video_id, video_metadata, error_message, loading).await;
                });
            }
            || ()
        });
    }

    html! {
        <div class="bg-gray-100 rounded-lg overflow-hidden">
            <div class="bg-gray-200 p-4 flex justify-between items-center cursor-pointer"
                 onclick={let expanded = expanded.clone(); move |_| expanded.set(!*expanded)}>
                <h3 class="text-lg font-semibold text-gray-800">
                    {"Video: "}
                    <a href={format!("https://www.youtube.com/watch?v={}", props.video_id)}
                       target="_blank"
                       class="text-blue-600 hover:underline">
                        { if let Some(metadata) = &props.metadata {
                            &metadata.title
                        } else {
                            &props.video_id
                        }}
                    </a>
                </h3>
                <span class="text-gray-600">
                    {if *expanded { "▼" } else { "▶" }}
                </span>
            </div>
            {
                if *expanded {
                    html! {
                        <div>
                            { if let Some(metadata) = &*video_metadata {
                                html! {
                                    <div class="bg-gray-50 p-4 text-sm">
                                        <p class="text-gray-600">{"📺 "}<span class="text-gray-900">{&metadata.channel_name}</span></p>
                                        <p class="text-gray-600">{"📅 "}<span class="text-gray-900">{&metadata.upload_date}</span></p>
                                        <p class="text-gray-600">{"⏱️ "}<span class="text-gray-900">{&metadata.duration}</span></p>
                                        <p class="text-gray-600">{"👁️ "}<span class="text-gray-900">{metadata.views}</span></p>
                                        <p class="text-gray-600">{"👍 "}<span class="text-gray-900">{metadata.likes}</span></p>
                                        <p class="text-gray-600">{"💬 "}<span class="text-gray-900">{metadata.comment_count}</span></p>
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                            <div class="divide-y divide-gray-200">
                                { for props.results.iter().map(|result| html! {
                                    <SearchResultItem result={result.clone()} />
                                })}
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

#[function_component(ResultsList)]
fn results_list(props: &ResultsListProps) -> Html {
    if props.results.is_empty()
        && !props.loading
        && props.error.is_none()
        && !props.query.is_empty()
    {
        return html! {
            <p class="text-center text-gray-500">{"No results found."}</p>
        };
    }

    let mut grouped_results: std::collections::HashMap<String, Vec<&SearchResult>> =
        std::collections::HashMap::new();
    for result in props.results.iter() {
        grouped_results
            .entry(result.video_id.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    html! {
        <div class="mt-8 space-y-6">
            { for grouped_results.iter().map(|(video_id, results)| {
                let mut sorted_results = results.iter().map(|&r| r.clone()).collect::<Vec<_>>();
                sorted_results.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
                html! {
                    <VideoResults
                        video_id={video_id.clone()}
                        results={sorted_results}
                        metadata={None}
                    />
                }
            })}
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ResultsListProps {
    pub results: Vec<SearchResult>,
    pub loading: bool,
    pub error: Option<String>,
    pub query: String,
}

async fn execute_search(
    query: String,
    search_results: UseStateHandle<Vec<SearchResult>>,
    error_message: UseStateHandle<Option<String>>,
    loading: UseStateHandle<bool>,
) {
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let url = format!("?q={}", query);
            let _ = history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&url));
        }
    }
    let response = perform_search_request(&query).await;
    handle_search_response(response, &search_results, &error_message).await;
    loading.set(false);
}

async fn get_video_metadata(
    video_id: String,
    video_metadata: UseStateHandle<Option<VideoMetadata>>,
    error_message: UseStateHandle<Option<String>>,
    loading: UseStateHandle<bool>,
) {
    let response = get_raw_video_metadata(&video_id).await;

    match response {
        Ok(response) => {
            if response.ok() {
                match response.json::<Option<VideoMetadata>>().await {
                    Ok(results) => video_metadata.set(results),
                    Err(e) => {
                        handle_error(&error_message, format!("Failed to parse video-id: {e}"))
                    }
                }
            } else {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                handle_error(
                    &error_message,
                    format!("Search failed: HTTP {status} - {text}"),
                );
            }
        }
        Err(e) => handle_error(&error_message, format!("Failed to connect to backend: {e}")),
    }

    loading.set(false);
}

async fn get_raw_video_metadata(
    video_id: &str,
) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let backend_url = "http://localhost:8000";
    let url = format!("{backend_url}/video/{video_id}");
    Request::get(&url).send().await
}

async fn perform_search_request(query: &str) -> Result<gloo_net::http::Response, gloo_net::Error> {
    let backend_url = "http://localhost:8000";
    let url = format!("{backend_url}/search?q={query}");
    Request::get(&url).send().await
}

async fn handle_search_response(
    response: Result<gloo_net::http::Response, gloo_net::Error>,
    search_results: &UseStateHandle<Vec<SearchResult>>,
    error_message: &UseStateHandle<Option<String>>,
) {
    match response {
        Ok(response) => {
            if response.ok() {
                match response.json::<Vec<SearchResult>>().await {
                    Ok(results) => search_results.set(results),
                    Err(e) => handle_error(
                        error_message,
                        format!("Failed to parse search results: {e}"),
                    ),
                }
            } else {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                handle_error(
                    error_message,
                    format!("Search failed: HTTP {status} - {text}"),
                );
            }
        }
        Err(e) => handle_error(error_message, format!("Failed to connect to backend: {e}")),
    }
}

fn handle_error(error_message: &UseStateHandle<Option<String>>, error: String) {
    error_message.set(Some(error.clone()));
    web_sys::console::error_1(&error.into());
}

#[function_component(App)]
pub fn app() -> Html {
    let search_query = use_state(String::default);
    let search_results = use_state(Vec::<SearchResult>::default);
    let loading = use_state(|| false);
    let error_message = use_state(Option::<String>::default);
    let init_done = use_state(|| false);

    {
        let search_query = search_query.clone();
        let search_results = search_results.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let init_done = init_done.clone();

        use_effect(move || {
            if !*init_done {
                if let Some(query) = get_query_param() {
                    search_query.set(query.clone());
                    loading.set(true);
                    error_message.set(None);

                    wasm_bindgen_futures::spawn_local(async move {
                        execute_search(query, search_results, error_message, loading).await;
                    });
                }
                init_done.set(true);
            }
            || ()
        });
    }

    // Callback for input change
    let on_input = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value());
        })
    };

    // Callback for the search button click
    let on_search = {
        let search_query = search_query.clone();
        let search_results = search_results.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();

        // Change the callback to accept a MouseEvent, as required by onclick
        Callback::from(move |_e: MouseEvent| {
            // Added _e: MouseEvent
            let query = (*search_query).clone();
            let search_results = search_results.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();

            loading.set(true);
            error_message.set(None); // Clear previous errors

            wasm_bindgen_futures::spawn_local(async move {
                execute_search(query, search_results, error_message, loading).await;
            });
        })
    };

    html! {
        <div class="min-h-screen flex flex-col items-center justify-center bg-gray-700 p-4">
            <div class="bg-white p-8 rounded-lg shadow-lg w-full max-w-2xl">
                <h1 class="text-3xl font-bold text-center text-gray-800 mb-6">
                    {"YouTube Caption Search"}
                </h1>

                <SearchBar
                    query={(*search_query).clone()}
                    loading={*loading}
                    on_input={on_input}
                    on_search={on_search.clone()}
                    on_enter={
                        let on_search_clone = on_search.clone();
                        Callback::from(move |e: KeyboardEvent| {
                            if e.key() == "Enter" {
                                on_search_clone.emit(MouseEvent::new("click").unwrap());
                            }
                        })
                    }
                />

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
                />
            </div>
        </div>
    }
}

fn get_query_param() -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get("q")
}

// Entry point for the Yew app
fn main() {
    yew::Renderer::<App>::new().render();
}
