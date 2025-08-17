use crate::models::SearchResult;
use crate::search::api::get_video_metadata;
use crate::utils::{format_iso8601_date, format_iso8601_duration, format_number, format_timestamp};
use web_sys::HtmlInputElement;
use yew::prelude::*;

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

const RESULTS_PER_PAGE: usize = 10;

#[derive(Properties, PartialEq)]
pub struct ResultsListProps {
    pub results: Vec<SearchResult>,
    pub loading: bool,
    pub error: Option<String>,
    pub query: String,
    pub on_page_change: Callback<usize>,
    pub current_page: usize,
    pub total_results: Option<(usize, usize)>, // (total_videos, total_captions)
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
pub fn search_result_item(props: &SearchResultItemProps) -> Html {
    html! {
        <div class="p-4 bg-white">
            <p class="text-sm text-gray-500 mb-1">
                <a href={format!("https://www.youtube.com/watch?v={}&t={}s", props.result.video_id, props.result.start_time)}
                   target="_blank"
                   class="ml-2 text-blue-600 hover:underline">
                {format!("{} ↗ ", format_timestamp(props.result.start_time))}
                </a>
            { Html::from_html_unchecked(AttrValue::from(props.result.snippet_html.clone())) }
            </p>
        </div>
    }
}

#[function_component(VideoResults)]
pub fn video_results(props: &VideoResultsProps) -> Html {
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
pub fn results_list(props: &ResultsListProps) -> Html {
    if props.results.is_empty()
        && !props.loading
        && props.error.is_none()
        && !props.query.is_empty()
    {
        return html! {
            <p class="text-center text-gray-500">{"No results found."}</p>
        };
    }

    // Calculate total pages based on total videos (not captions)
    let total_pages = if let Some((total_videos, _)) = props.total_results {
        (total_videos as f32 / RESULTS_PER_PAGE as f32).ceil() as usize
    } else {
        1
    };

    let mut last_video = String::new();
    let mut current_group = Vec::new();
    let mut grouped_videos = Vec::new();

    // Group results while preserving video order from backend
    for result in props.results.iter() {
        if result.video_id != last_video {
            if !current_group.is_empty() {
                grouped_videos.push((last_video.clone(), current_group));
                current_group = Vec::new();
            }
            last_video = result.video_id.clone();
        }
        current_group.push(result);
    }
    if !current_group.is_empty() {
        grouped_videos.push((last_video, current_group));
    }

    html! {
        <div class="mt-8">
            // Add results summary
            {
                if let Some((total_videos, total_captions)) = props.total_results {
                    html! {
                        <div class="mb-4 p-3 bg-blue-50 rounded-lg text-center">
                            <p class="text-sm text-gray-700">
                                {format!("Found {} matching videos with {} total caption matches for \"{}\"",
                                    total_videos, total_captions, props.query)}
                            </p>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            <div class="space-y-6">
                { for grouped_videos.clone().into_iter().map(|(video_id, results)| {
                    let mut sorted_results = results.iter().map(|&r| r.clone()).collect::<Vec<_>>();
                    sorted_results.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
                    html! {
                        <VideoResults
                            video_id={video_id}
                            results={sorted_results}
                        />
                    }
                })}
            </div>

            // Update pagination to show more detailed info
            <div class="mt-6 flex flex-col items-center gap-2">
                <div class="flex justify-center gap-2">
                    <button
                        onclick={
                            let on_page_change = props.on_page_change.clone();
                            let current_page = props.current_page;
                            move |_| {
                                if current_page > 0 {
                                    on_page_change.emit(current_page - 1);
                                }
                                if let Some(window) = web_sys::window() {
                                    window.scroll_to_with_x_and_y(0.0, 0.0);
                                }
                            }
                        }
                        disabled={props.current_page == 0 || props.loading}
                        class="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
                    >
                        {"Previous"}
                    </button>
                    <span class="px-4 py-2 text-sm">
                        {format!("Page {} of {}", props.current_page + 1, total_pages.max(1))}
                    </span>
                    <button
                        onclick={
                            let on_page_change = props.on_page_change.clone();
                            let current_page = props.current_page;
                            move |_| {
                                if current_page < total_pages.saturating_sub(1) {
                                    on_page_change.emit(current_page + 1);
                                }
                                if let Some(window) = web_sys::window() {
                                    window.scroll_to_with_x_and_y(0.0, 0.0);
                                }
                            }
                        }
                        disabled={props.current_page >= total_pages.saturating_sub(1) || props.loading}
                        class="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
                    >
                        {"Next"}
                    </button>
                </div>
                {
                    if let Some((total_videos, _)) = props.total_results {
                        html! {
                            <p class="text-xs text-gray-500">
                                {format!("Showing {} videos", grouped_videos.len())}
                            </p>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        </div>
    }
}
