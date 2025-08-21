use crate::env_variable_utils::BACKEND_URL;
use crate::router::Route;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::{window, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct QueueItem {
    pub id: String,
    pub status: String,
    pub added_at: String,
    pub processed_at: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddUrlRequest {
    pub url: String,
}

#[derive(Properties, PartialEq)]
pub struct AdminQueuePageProps {}

#[function_component(AdminQueuePage)]
pub fn admin_queue_page(_props: &AdminQueuePageProps) -> Html {
    let queue_items = use_state(Vec::<QueueItem>::new);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let success_message = use_state(|| None::<String>);
    let new_url = use_state(String::new);

    // Load queue items on component mount
    {
        let queue_items = queue_items.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();

        use_effect_with((), move |_| {
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match load_queue_items().await {
                    Ok(items) => {
                        queue_items.set(items);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to load queue: {}", e)));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_url_input = {
        let new_url = new_url.clone();
        Callback::from(move |e: InputEvent| {
            let input_value = e.target_unchecked_into::<HtmlInputElement>().value();
            new_url.set(input_value);
        })
    };

    let on_add_url = {
        let new_url = new_url.clone();
        let queue_items = queue_items.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();

        Callback::from(move |e: web_sys::SubmitEvent| {
            e.prevent_default();

            // Clear previous messages
            error_message.set(None);
            success_message.set(None);

            let url = (*new_url).clone();
            if url.is_empty() {
                error_message.set(Some("Please enter a URL".to_string()));
                return;
            }

            let new_url = new_url.clone();
            let queue_items = queue_items.clone();
            let error_message = error_message.clone();
            let success_message = success_message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match add_url_to_queue(&url).await {
                    Ok(_) => {
                        new_url.set(String::new());
                        success_message.set(Some("URL added to queue successfully!".to_string()));
                        // Reload queue items
                        match load_queue_items().await {
                            Ok(items) => {
                                queue_items.set(items);
                            }
                            Err(e) => {
                                error_message.set(Some(format!("Failed to reload queue: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to add URL: {}", e)));
                    }
                }
            });
        })
    };

    let on_delete_item = {
        let queue_items = queue_items.clone();
        let error_message = error_message.clone();
        let success_message = success_message.clone();

        Callback::from(move |item_id: String| {
            let queue_items = queue_items.clone();
            let error_message = error_message.clone();
            let success_message = success_message.clone();

            // Clear previous messages
            error_message.set(None);
            success_message.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match delete_queue_item(&item_id).await {
                    Ok(_) => {
                        success_message.set(Some("Item deleted successfully!".to_string()));
                        // Remove item from list
                        let current_items = (*queue_items).clone();
                        let updated_items: Vec<QueueItem> = current_items
                            .into_iter()
                            .filter(|i| i.id != item_id)
                            .collect();
                        queue_items.set(updated_items);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to delete item: {}", e)));
                    }
                }
            });
        })
    };

    html! {
        <div class="min-h-screen bg-gray-700 p-4">
            <div class="max-w-6xl mx-auto">
                <div class="bg-white rounded-lg shadow-lg p-8">
                    <div class="flex justify-between items-center mb-6">
                        <h1 class="text-3xl font-bold text-gray-800">
                            {"Download Queue"}
                        </h1>
                        <Link<Route> to={Route::Admin} classes="text-blue-600 hover:underline">
                            {"← Back to Overview"}
                        </Link<Route>>
                    </div>

                    {
                        if let Some(msg) = &*success_message {
                            html! {
                                <div class="bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded mb-4">
                                    { msg }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }

                    {
                        if let Some(msg) = &*error_message {
                            html! {
                                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                                    { msg }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }

                    // Add URL form
                    <div class="mb-6 bg-gray-50 p-4 rounded-lg">
                        <h3 class="text-lg font-semibold text-gray-800 mb-4">{"Add URL to Queue"}</h3>
                        <form onsubmit={on_add_url} class="flex gap-4">
                            <input
                                type="url"
                                class="flex-1 p-3 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                placeholder="Enter YouTube URL..."
                                value={(*new_url).clone()}
                                oninput={on_url_input}
                            />
                            <button
                                type="submit"
                                class="bg-blue-600 text-white px-6 py-3 rounded hover:bg-blue-700"
                            >
                                {"Add to Queue"}
                            </button>
                        </form>
                    </div>

                    {
                        if *loading {
                            html! {
                                <div class="text-center py-8">
                                    <p>{"Loading queue..."}</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="overflow-x-auto">
                                    <table class="min-w-full bg-white border border-gray-300">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Status"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Added"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Processed"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Actions"}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {
                                                (*queue_items).iter().map(|item| {
                                                    let item_id = item.id.clone();
                                                    let on_delete = on_delete_item.clone();

                                                    html! {
                                                        <tr key={item.id.clone()}>
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                <span class={format!("px-2 inline-flex text-xs leading-5 font-semibold rounded-full {}",
                                                                    match item.status.as_str() {
                                                                        "pending" => "bg-yellow-100 text-yellow-800",
                                                                        "processing" => "bg-blue-100 text-blue-800",
                                                                        "completed" => "bg-green-100 text-green-800",
                                                                        "failed" => "bg-red-100 text-red-800",
                                                                        _ => "bg-gray-100 text-gray-800"
                                                                    }
                                                                )}>
                                                                    {&item.status}
                                                                </span>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {&item.added_at}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {item.processed_at.as_deref().unwrap_or("N/A")}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                                                                <button
                                                                    onclick={
                                                                        let item_id = item_id.clone();
                                                                        let on_delete = on_delete.clone();
                                                                        Callback::from(move |_| {
                                                                            on_delete.emit(item_id.clone());
                                                                        })
                                                                    }
                                                                    class="text-red-600 hover:text-red-900"
                                                                >
                                                                    {"Delete"}
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect::<Html>()
                                            }
                                        </tbody>
                                    </table>
                                </div>
                            }
                        }
                    }
                </div>
            </div>
        </div>
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct QueueResponse {
    success: bool,
    message: String,
    items: Vec<QueueItem>,
}

async fn load_queue_items() -> Result<Vec<QueueItem>, String> {
    let backend_url = &*BACKEND_URL;
    let url = format!("{}/admin/queue", backend_url);

    let token = window()
        .and_then(|w| w.session_storage().ok())
        .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
        .flatten()
        .ok_or("No admin token found")?;

    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let queue_response = response
            .json::<QueueResponse>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;
        Ok(queue_response.items)
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn add_url_to_queue(url: &str) -> Result<(), String> {
    let backend_url = &*BACKEND_URL;
    let api_url = format!("{}/admin/queue", backend_url);

    let token = window()
        .and_then(|w| w.session_storage().ok())
        .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
        .flatten()
        .ok_or("No admin token found")?;

    let request_body = AddUrlRequest {
        url: url.to_string(),
    };

    let response = Request::post(&api_url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request_body)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn delete_queue_item(item_id: &str) -> Result<(), String> {
    let backend_url = &*BACKEND_URL;
    let url = format!("{}/admin/queue/{}", backend_url, item_id);

    let token = window()
        .and_then(|w| w.session_storage().ok())
        .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
        .flatten()
        .ok_or("No admin token found")?;

    let response = Request::delete(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}
