mod admin;
mod models;
mod router;

use crate::models::{SearchResult, VideoMetadata};
use crate::router::{switch, Route};
use gloo_net::http::Request;
use web_sys::console;
use web_sys::{wasm_bindgen, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

fn format_iso8601_date(iso_date: &str) -> String {
    if let Ok(datetime) = iso_date.parse::<chrono::DateTime<chrono::Utc>>() {
        datetime.format("%Y-%m-%d").to_string()
    } else {
        iso_date.to_string()
    }
}

// Formats each x1000 step
fn format_number(number: i64) -> String {
    let num_str = number.to_string();
    let mut result = String::new();
    let len = num_str.len();

    for (i, c) in num_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

fn format_timestamp(seconds: f64) -> String {
    let minutes = (seconds as u32) / 60;
    let remaining_seconds = (seconds as u32) % 60;
    format!("{:02}:{:02}", minutes, remaining_seconds)
}

fn format_iso8601_duration(duration: &str) -> String {
    let hours = duration
        .find('H')
        .map_or(0, |h| duration[2..h].parse::<u32>().unwrap_or(0));
    let minutes = duration.find('M').map_or(0, |m| {
        let start = duration.find('H').map_or(2, |h| h + 1);
        duration[start..m].parse::<u32>().unwrap_or(0)
    });
    let seconds = duration.find('S').map_or(0, |s| {
        let start = duration
            .find('M')
            .map_or_else(|| duration.find('H').map_or(2, |h| h + 1), |m| m + 1);
        duration[start..s].parse::<u32>().unwrap_or(0)
    });
    if hours != 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

async fn execute_search(
    query: String,
    search_results: UseStateHandle<Vec<SearchResult>>,
    error_message: UseStateHandle<Option<String>>,
    loading: UseStateHandle<bool>,
) {
    // Update URL with query parameter
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let url = format!("/?q={}", query);
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

fn handle_error(error_message: &UseStateHandle<Option<String>>, error: String) {
    error_message.set(Some(error.clone()));
    web_sys::console::error_1(&error.into());
}

fn get_query_param() -> Option<String> {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|search| web_sys::UrlSearchParams::new_with_str(&search).ok())
        .and_then(|params| {
            let result = params.get("q");
            match &result {
                Some(val) => console::log_1(&format!("query-param: {}", val).into()),
                None => console::log_1(&"query-param: Not found".into()),
            }
            result
        })
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

#[derive(Properties, PartialEq)]
pub struct SearchBarProps {
    pub query: String,
    pub loading: bool,
    pub on_search: Callback<String>,
}

#[derive(Properties, PartialEq)]
pub struct SearchResultItemProps {
    pub result: SearchResult,
}

#[derive(Properties, PartialEq)]
pub struct VideoResultsProps {
    pub video_id: String,
    pub results: Vec<SearchResult>,
}

#[derive(Properties, PartialEq)]
pub struct ResultsListProps {
    pub results: Vec<SearchResult>,
    pub loading: bool,
    pub error: Option<String>,
    pub query: String,
}

#[function_component(SearchBar)]
pub fn search_bar(props: &SearchBarProps) -> Html {
    let current_input = use_state(|| props.query.clone());

    // This Callback handles when the user types into the input field.
    let on_input = {
        let current_input = current_input.clone();
        Callback::from(move |e: InputEvent| {
            let input_value = e.target_unchecked_into::<HtmlInputElement>().value();
            current_input.set(input_value);
        })
    };

    // This Callback handles form submission.
    let on_submit = {
        let on_search = props.on_search.clone();
        let current_input = current_input.clone();
        Callback::from(move |e: web_sys::SubmitEvent| {
            e.prevent_default(); // Prevent default form submission (page reload)
            on_search.emit((*current_input).clone()); // Emit the current value to the parent
        })
    };

    html! {
        <form onsubmit={on_submit} class="flex mb-4">
            <input
                type="text"
                class="flex-grow p-3 border border-gray-300 rounded-l-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Enter YouTube caption search query..."
                value={(*current_input).clone()} // Bind the input's value to our internal state
                oninput={on_input} // Update internal state on user input
                disabled={props.loading}
            />
            <button
                type="submit"
                class="bg-blue-600 text-white p-3 rounded-r-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                disabled={props.loading}
            >
                { if props.loading { "Searching..." } else { "Search" } }
            </button>
        </form>
    }
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
                    <a href={format!("https://www.youtube.com/watch?v={}", props.video_id)}
                       target="_blank"
                       class="text-blue-600 hover:underline">
                        { if let Some(metadata) = &*video_metadata {
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
                                    <div class="bg-gray-50 p-4 text-sm flex flex-wrap gap-4">
                                        <p class="flex items-center">{"📺 "}<a href={format!("https://www.youtube.com/channel/{}",&metadata.channel_id)} class="text-blue-600 hover:underline">{&metadata.channel_name}</a></p>
                                        <p class="flex items-center">{"📅 "}<span>{format_iso8601_date(&metadata.upload_date)}</span></p>
                                        <p class="flex items-center">{"⏱️ "}<span>{format_iso8601_duration(&metadata.duration)}</span></p>
                                        <p class="flex items-center">{"👁️ "}<span>{format_number(metadata.views)}</span></p>
                                        <p class="flex items-center">{"👍 "}<span>{format_number(metadata.likes)}</span></p>
                                        <p class="flex items-center">{"💬 "}<span>{format_number(metadata.comment_count)}</span></p>
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
                    />
                }
            })}
        </div>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

// Entry point for the Yew app
fn main() {
    yew::Renderer::<App>::new().render();
}
