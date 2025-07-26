use crate::router::Route;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub subscriber_count: Option<i64>,
    pub video_count: Option<i64>,
    pub last_crawled: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct AdminChannelsPageProps {}

#[function_component(AdminChannelsPage)]
pub fn admin_channels_page(_props: &AdminChannelsPageProps) -> Html {
    let channels = use_state(Vec::<Channel>::new);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);

    // Load channels on component mount
    {
        let channels = channels.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();

        use_effect_with((), move |_| {
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match load_channels().await {
                    Ok(channel_list) => {
                        channels.set(channel_list);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to load channels: {}", e)));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_delete_channel = {
        let channels = channels.clone();
        let error_message = error_message.clone();

        Callback::from(move |channel_id: String| {
            let channels = channels.clone();
            let error_message = error_message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match delete_channel(&channel_id).await {
                    Ok(_) => {
                        // Remove channel from list
                        let current_channels = (*channels).clone();
                        let updated_channels: Vec<Channel> = current_channels
                            .into_iter()
                            .filter(|c| c.id != channel_id)
                            .collect();
                        channels.set(updated_channels);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to delete channel: {}", e)));
                    }
                }
            });
        })
    };

    let on_trigger_crawl = {
        let error_message = error_message.clone();

        Callback::from(move |channel_id: String| {
            let error_message = error_message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match trigger_channel_crawl(&channel_id).await {
                    Ok(_) => {
                        error_message
                            .set(Some("Channel crawl triggered successfully!".to_string()));
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to trigger crawl: {}", e)));
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
                            {"Channels"}
                        </h1>
                        <Link<Route> to={Route::Admin} classes="text-blue-600 hover:underline">
                            {"← Back to Overview"}
                        </Link<Route>>
                    </div>

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

                    {
                        if *loading {
                            html! {
                                <div class="text-center py-8">
                                    <p>{"Loading channels..."}</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="overflow-x-auto">
                                    <table class="min-w-full bg-white border border-gray-300">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Name"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Subscribers"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Videos"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Last Crawled"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Actions"}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {
                                                (*channels).iter().map(|channel| {
                                                    let channel_id = channel.id.clone();
                                                    let on_delete = on_delete_channel.clone();
                                                    let on_crawl = on_trigger_crawl.clone();

                                                    html! {
                                                        <tr key={channel.id.clone()}>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                <div class="max-w-xs truncate">{&channel.name}</div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {channel.subscriber_count.map(|v| v.to_string()).unwrap_or_else(|| "N/A".to_string())}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {channel.video_count.map(|v| v.to_string()).unwrap_or_else(|| "N/A".to_string())}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {channel.last_crawled.as_deref().unwrap_or("Never")}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                                                                <button
                                                                    onclick={
                                                                        let channel_id = channel_id.clone();
                                                                        let on_crawl = on_crawl.clone();
                                                                        Callback::from(move |_| {
                                                                            on_crawl.emit(channel_id.clone());
                                                                        })
                                                                    }
                                                                    class="text-blue-600 hover:text-blue-900 mr-4"
                                                                >
                                                                    {"Crawl"}
                                                                </button>
                                                                <button
                                                                    onclick={
                                                                        let channel_id = channel_id.clone();
                                                                        let on_delete = on_delete.clone();
                                                                        Callback::from(move |_| {
                                                                            on_delete.emit(channel_id.clone());
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

async fn load_channels() -> Result<Vec<Channel>, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/channels", backend_url);

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
        response
            .json::<Vec<Channel>>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn delete_channel(channel_id: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/channels/{}", backend_url, channel_id);

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

async fn trigger_channel_crawl(channel_id: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/channels/{}/crawl", backend_url, channel_id);

    let token = window()
        .and_then(|w| w.session_storage().ok())
        .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
        .flatten()
        .ok_or("No admin token found")?;

    let response = Request::post(&url)
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
