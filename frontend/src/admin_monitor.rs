use crate::models::{MonitoredChannel, MonitoredChannelModify};
use crate::router::Route;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AdminChannelsPageProps {}

#[function_component(AdminChannelsPage)]
pub fn admin_channels_page(_props: &AdminChannelsPageProps) -> Html {
    let channels = use_state(Vec::<MonitoredChannel>::new);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let error_message = use_state(|| None::<String>);
    let new_channel_id = use_state(|| String::new());

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
                        let updated_channels: Vec<MonitoredChannel> = current_channels
                            .into_iter()
                            .filter(|c| c.channel_id != channel_id)
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

                    <div class="mb-6">
                        <form class="flex gap-4"
                            onsubmit={
                                let new_channel_id = new_channel_id.clone();
                                let channels = channels.clone();
                                let error_message = error_message.clone();

                                Callback::from(move |e: SubmitEvent| {
                                    e.prevent_default();
                                    let channel_id = (*new_channel_id).clone();
                                    let channels = channels.clone();
                                    let error_message = error_message.clone();
                                    let new_channel_id = new_channel_id.clone();

                                    wasm_bindgen_futures::spawn_local(async move {
                                        match add_channel(&channel_id).await {
                                            Ok(_) => {
                                                match load_channels().await {
                                                    Ok(channel_list) => {
                                                        channels.set(channel_list);
                                                        new_channel_id.set(String::new());
                                                    }
                                                    Err(e) => {
                                                        error_message.set(Some(format!("Failed to reload channels: {}", e)));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error_message.set(Some(format!("Failed to add channel: {}", e)));
                                            }
                                        }
                                    });
                                })
                            }
                        >
                            <input
                                type="text"
                                placeholder="Enter YouTube Channel ID"
                                class="flex-grow px-4 py-2 border rounded"
                                value={(*new_channel_id).clone()}
                                onchange={
                                    let new_channel_id = new_channel_id.clone();
                                    Callback::from(move |e: Event| {
                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                        new_channel_id.set(input.value());
                                    })
                                }
                            />
                            <button
                                type="submit"
                                class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                            >
                                {"Add Channel"}
                            </button>
                        </form>
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
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Videos"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Active"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Actions"}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {
                                                (*channels).iter().map(|channel| {
                                                    let channel_id = channel.channel_id.clone();
                                                    let on_delete = on_delete_channel.clone();
                                                    let channel_link = format!("https://www.youtube.com/channel/{}", &channel.channel_id);

                                                    html! {
                                                        <tr>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                <div class="max-w-xs truncate"><a href={format!("https://www.youtube.com/channel/{}",&channel.channel_id)} class="text-blue-600 hover:underline">{&channel.channel_id}</a></div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {"TODO"}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {if channel.active { "Yes" } else { "No" }}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                                                                <div class="flex gap-2">
                                                                    <button
                                                                        onclick={
                                                                            let channel_id = channel_id.clone();
                                                                            let error_message = error_message.clone();
                                                                            Callback::from(move |_| {
                                                                                let channel_id = channel_id.clone();
                                                                                let error_message = error_message.clone();
                                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                                    if let Err(e) = force_check_complete_channel(&channel_id).await {
                                                                                        error_message.set(Some(format!("Failed to check channel: {}", e)));
                                                                                    }
                                                                                });
                                                                            })
                                                                        }
                                                                        class="text-blue-600 hover:text-blue-900"
                                                                    >
                                                                        {"Check"}
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
                                                                </div>
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

async fn load_channels() -> Result<Vec<MonitoredChannel>, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/channel", backend_url);

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
            .json::<Vec<MonitoredChannel>>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

fn extract_channel_id(input: &str) -> String {
    if let Some(channel_id) = input.strip_prefix("https://www.youtube.com/channel/") {
        channel_id.split('/').next().unwrap_or(input).to_string()
    } else {
        input.to_string()
    }
}

async fn add_channel(input: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/channel", backend_url);

    let token = window()
        .and_then(|w| w.session_storage().ok())
        .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
        .flatten()
        .ok_or("No admin token found")?;

    let channel_id = extract_channel_id(input);

    let channel = MonitoredChannelModify {
        channel_id: channel_id,
        channel_name: "".to_string(), // Will be populated by backend
        active: true,
    };

    let response = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .json(&channel)
        .map_err(|e| format!("Failed to serialize: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn delete_channel(channel_id: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/channel/{}", backend_url, channel_id);

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

async fn force_check_complete_channel(channel_id: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/channel/{}/check", backend_url, channel_id);

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
