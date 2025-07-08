use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use web_sys::MouseEvent;
use yew::prelude::*; // Import MouseEvent

// Data models for frontend
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub video_id: String,
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub highlighted_text: Option<String>,
}

// Main App component
#[function_component(App)]
pub fn app() -> Html {
    let search_query = use_state(String::default);
    let search_results = use_state(Vec::<SearchResult>::default);
    let loading = use_state(|| false);
    let error_message = use_state(Option::<String>::default);

    // Callback for input change
    let on_input = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value());
        })
    };

    // Callback for search button click
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
                let backend_url = "http://localhost:8000"; // Rocket backend URL
                let url = format!("{backend_url}/search?q={query}");

                match Request::get(&url).send().await {
                    Ok(response) => {
                        if response.ok() {
                            match response.json::<Vec<SearchResult>>().await {
                                Ok(results) => {
                                    search_results.set(results);
                                }
                                Err(e) => {
                                    error_message
                                        .set(Some(format!("Failed to parse search results: {e}")));
                                    web_sys::console::error_1(
                                        &format!("Failed to parse search results: {e}").into(),
                                    );
                                }
                            }
                        } else {
                            let status = response.status();
                            let text = response.text().await.unwrap_or_default();
                            error_message
                                .set(Some(format!("Search failed: HTTP {status} - {text}")));
                            web_sys::console::error_1(
                                &format!("Search failed: HTTP {status} - {text}").into(),
                            );
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to connect to backend: {e}")));
                        web_sys::console::error_1(
                            &format!("Failed to connect to backend: {e}").into(),
                        );
                    }
                }
                loading.set(false);
            });
        })
    };

    html! {
        <div class="min-h-screen flex flex-col items-center justify-center bg-gray-700 p-4">
            <div class="bg-white p-8 rounded-lg shadow-lg w-full max-w-2xl">
                <h1 class="text-3xl font-bold text-center text-gray-800 mb-6">
                    {"YouTube Caption Search"}
                </h1>

                <div class="flex flex-col sm:flex-row gap-4 mb-6">
                    <input
                        type="text"
                        class="flex-grow p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        placeholder="Enter search query..."
                        value={(*search_query).clone()}
                        oninput={on_input}
                        onkeydown={
                            let on_search_clone = on_search.clone();
                            Callback::from(move |e: KeyboardEvent| {
                                if e.key() == "Enter" {
                                    // When pressing Enter, we also need to pass a dummy MouseEvent
                                    // or adjust the on_search callback to accept a generic event or no event.
                                    // For simplicity and to match the onclick signature, we'll create a dummy MouseEvent.
                                    // A better approach for shared logic is to extract the search logic into a separate function
                                    // that both callbacks can call.
                                    on_search_clone.emit(MouseEvent::new("click").unwrap()); // Emit a dummy click event
                                }
                            })
                        }
                    />
                    <button
                        class="px-6 py-3 bg-blue-600 text-white font-semibold rounded-md shadow-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 transition duration-200 ease-in-out"
                        onclick={on_search}
                        disabled={*loading}
                    >
                        { if *loading { "Searching..." } else { "Search" } }
                    </button>
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

                <div class="mt-8 space-y-4">
                    {
                        if search_results.is_empty() && !*loading && error_message.is_none() && !search_query.is_empty() {
                            html! {
                                <p class="text-center text-gray-500">{"No results found."}</p>
                            }
                        } else {
                            html! {
                                for search_results.iter().map(|result| html! {
                                    <div class="bg-gray-50 p-4 rounded-lg shadow-sm border border-gray-200">
                                        <p class="text-sm text-gray-500 mb-1">
                                            {"Video ID: "}
                                            <a href={format!("https://www.youtube.com/watch?v={}", result.video_id)} target="_blank" class="text-blue-600 hover:underline">
                                                { &result.video_id }
                                            </a>
                                            {format!(" ({}s - {}s)", result.start_time as u32, result.end_time as u32)}
                                        </p>
                                        <p class="text-gray-800 leading-relaxed">
                                            {
                                                if let Some(highlight) = &result.highlighted_text {
                                                    Html::from_html_unchecked(AttrValue::from(highlight.clone()))
                                                } else {
                                                    html! { &result.text }
                                                }
                                            }
                                        </p>
                                    </div>
                                })
                            }
                        }
                    }
                </div>
            </div>
        </div>
    }
}

// Entry point for the Yew app
fn main() {
    yew::Renderer::<App>::new().render();
}
