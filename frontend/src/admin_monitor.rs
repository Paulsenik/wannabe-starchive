use crate::models::{MonitoredChannelStats, MonitoredPlaylistStats};
use crate::router::Route;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AdminChannelsPageProps {}

#[derive(Serialize, Deserialize, Clone)]
pub struct MonitoredChannelModify {
    pub channel_input: String,
    pub channel_name: String,
    pub active: bool,
}

#[function_component(AdminMonitorsPage)]
pub fn admin_monitors_page(_props: &AdminChannelsPageProps) -> Html {
    let channels = use_state(Vec::<MonitoredChannelStats>::new);
    let playlists = use_state(Vec::<MonitoredPlaylistStats>::new);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let new_channel_id = use_state(|| String::new());
    let new_playlist_id = use_state(|| String::new());

    // Load channels on component mount
    {
        let channels = channels.clone();
        let playlists = playlists.clone();
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

                match load_playlists().await {
                    Ok(playlist_list) => {
                        playlists.set(playlist_list);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to load playlists: {}", e)));
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
                        let updated_channels: Vec<MonitoredChannelStats> = current_channels
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
                            {"Monitors"}
                        </h1>
                        <Link<Route> to={Route::Admin} classes="text-blue-600 hover:underline">
                            {"← Back to Overview"}
                        </Link<Route>>
                    </div>
                    <div class="bg-white rounded-lg shadow-lg p-8 mt-8">
                        <h2 class="text-3xl font-bold text-gray-800">
                            {"Channels"}
                        </h2>

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
                                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Indexed Videos"}</th>
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
                                                                    <div class="max-w-xs truncate"><a href={format!("https://www.youtube.com/channel/{}",&channel.channel_id)} class="text-blue-600 hover:underline">{&channel.channel_name}</a></div>
                                                                </td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                    {&channel.videos_indexed}
                                                                    {" / "}
                                                                    {&channel.videos_uploaded}
                                                                </td>
                                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                    <button
                                                                        onclick={
                                                                            let channel_id = channel.channel_id.clone();
                                                                            let current_active = channel.active;
                                                                            let channels = channels.clone();
                                                                            let error_message = error_message.clone();

                                                                            Callback::from(move |_| {
                                                                                let channel_id = channel_id.clone();
                                                                                let channels = channels.clone();
                                                                                let error_message = error_message.clone();

                                                                                wasm_bindgen_futures::spawn_local(async move {
                                                                                    match toggle_channel_active(&channel_id, !current_active).await {
                                                                                        Ok(_) => {
                                                                                            match load_channels().await {
                                                                                                Ok(channel_list) => {
                                                                                                    channels.set(channel_list);
                                                                                                }
                                                                                                Err(e) => {
                                                                                                    error_message.set(Some(format!("Failed to reload channels: {}", e)));
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                        Err(e) => {
                                                                                            error_message.set(Some(format!("Failed to toggle channel status: {}", e)));
                                                                                        }
                                                                                    }
                                                                                });
                                                                            })
                                                                        }
                                                                        class={if channel.active {
                                                                            "px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
                                                                        } else {
                                                                            "px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
                                                                        }}
                                                                    >
                                                                        {if channel.active { "Active" } else { "Inactive" }}
                                                                    </button>
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
                    <div class="bg-white rounded-lg shadow-lg p-8 mt-8">
                        <h2 class="text-3xl font-bold text-gray-800 mb-6">{"Playlists"}</h2>
                        <div class="mb-6">
                            <form class="flex gap-4"
                                onsubmit={
                                    let new_playlist_id = new_playlist_id.clone();
                                    let playlists = playlists.clone();
                                    let error_message = error_message.clone();

                                    Callback::from(move |e: SubmitEvent| {
                                        e.prevent_default();
                                        let playlist_id = (*new_playlist_id).clone();
                                        let playlists = playlists.clone();
                                        let error_message = error_message.clone();
                                        let new_playlist_id = new_playlist_id.clone();

                                        wasm_bindgen_futures::spawn_local(async move {
                                            match add_playlist(&playlist_id).await {
                                                Ok(_) => {
                                                    match load_playlists().await {
                                                        Ok(playlist_list) => {
                                                            playlists.set(playlist_list);
                                                            new_playlist_id.set(String::new());
                                                        }
                                                        Err(e) => {
                                                            error_message.set(Some(format!("Failed to reload playlists: {}", e)));
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    error_message.set(Some(format!("Failed to add playlist: {}", e)));
                                                }
                                            }
                                        });
                                    })
                                }
                            >
                                <input
                                    type="text"
                                    placeholder="Enter YouTube Playlist ID"
                                    class="flex-grow px-4 py-2 border rounded"
                                    value={(*new_playlist_id).clone()}
                                    onchange={
                                        let new_playlist_id = new_playlist_id.clone();
                                        Callback::from(move |e: Event| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            new_playlist_id.set(input.value());
                                        })
                                    }
                                />
                                <button
                                    type="submit"
                                    class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                                >
                                    {"Add Playlist"}
                                </button>
                            </form>
                        </div>

                        <div class="overflow-x-auto">
                            <table class="min-w-full bg-white border border-gray-300">
                                <thead class="bg-gray-50">
                                    <tr>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Name"}</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Indexed Videos"}</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Active"}</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Actions"}</th>
                                    </tr>
                                </thead>
                                <tbody class="bg-white divide-y divide-gray-200">
                                    {
                                        (*playlists).iter().map(|playlist| {
                                            let playlist_id = playlist.playlist_id.clone();
                                            let playlist_link = format!("https://www.youtube.com/playlist?list={}", &playlist.playlist_id);

                                            html! {
                                                <tr>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                        <div class="max-w-xs truncate"><a href={playlist_link} class="text-blue-600 hover:underline">{&playlist.playlist_name}</a></div>
                                                    </td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                        {&playlist.videos_indexed}
                                                        {" / "}
                                                        {&playlist.videos_added}
                                                    </td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                        <button
                                                            onclick={
                                                                let playlist_id = playlist.playlist_id.clone();
                                                                let current_active = playlist.active;
                                                                let playlists = playlists.clone();
                                                                let error_message = error_message.clone();

                                                                Callback::from(move |_| {
                                                                    let playlist_id = playlist_id.clone();
                                                                    let playlists = playlists.clone();
                                                                    let error_message = error_message.clone();

                                                                    wasm_bindgen_futures::spawn_local(async move {
                                                                        match toggle_playlist_active(&playlist_id, !current_active).await {
                                                                            Ok(_) => {
                                                                                match load_playlists().await {
                                                                                    Ok(playlist_list) => {
                                                                                        playlists.set(playlist_list);
                                                                                    }
                                                                                    Err(e) => {
                                                                                        error_message.set(Some(format!("Failed to reload playlists: {}", e)));
                                                                                    }
                                                                                }
                                                                            }
                                                                            Err(e) => {
                                                                                error_message.set(Some(format!("Failed to toggle playlist status: {}", e)));
                                                                            }
                                                                        }
                                                                    });
                                                                })
                                                            }
                                                            class={if playlist.active {
                                                                "px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
                                                            } else {
                                                                "px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
                                                            }}
                                                        >
                                                            {if playlist.active { "Active" } else { "Inactive" }}
                                                        </button>
                                                    </td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                                                        <div class="flex gap-2">
                                                            <button
                                                                onclick={
                                                                    let playlist_id = playlist_id.clone();
                                                                    let error_message = error_message.clone();
                                                                    Callback::from(move |_| {
                                                                        let playlist_id = playlist_id.clone();
                                                                        let error_message = error_message.clone();
                                                                        wasm_bindgen_futures::spawn_local(async move {
                                                                            if let Err(e) = force_check_complete_playlist(&playlist_id).await {
                                                                                error_message.set(Some(format!("Failed to check playlist: {}", e)));
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
                                                                    let playlist_id = playlist_id.clone();
                                                                    let playlists = playlists.clone();
                                                                    let error_message = error_message.clone();
                                                                    Callback::from(move |_| {
                                                                        let playlist_id = playlist_id.clone();
                                                                        let playlists = playlists.clone();
                                                                        let error_message = error_message.clone();
                                                                        wasm_bindgen_futures::spawn_local(async move {
                                                                            match delete_playlist(&playlist_id).await {
                                                                                Ok(_) => {
                                                                                    let current_playlists = (*playlists).clone();
                                                                                    let updated_playlists: Vec<MonitoredPlaylistStats> = current_playlists
                                                                                        .into_iter()
                                                                                        .filter(|p| p.playlist_id != playlist_id)
                                                                                        .collect();
                                                                                    playlists.set(updated_playlists);
                                                                                }
                                                                                Err(e) => {
                                                                                    error_message.set(Some(format!("Failed to delete playlist: {}", e)));
                                                                                }
                                                                            }
                                                                        });
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
                    </div>
                </div>
            </div>
        </div>
    }
}

async fn load_channels() -> Result<Vec<MonitoredChannelStats>, String> {
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
            .json::<Vec<MonitoredChannelStats>>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewChannel {
    input: String,
}

async fn add_channel(input: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/channel", backend_url);

    let token = window()
        .and_then(|w| w.session_storage().ok())
        .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
        .flatten()
        .ok_or("No admin token found")?;

    let new_channel = NewChannel {
        input: input.to_string(),
    };

    let response = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&new_channel)
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

async fn load_playlists() -> Result<Vec<MonitoredPlaylistStats>, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/playlist", backend_url);

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
            .json::<Vec<MonitoredPlaylistStats>>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn add_playlist(input: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/playlist", backend_url);

    let token = window()
        .and_then(|w| w.session_storage().ok())
        .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
        .flatten()
        .ok_or("No admin token found")?;

    let new_playlist = NewChannel {
        input: input.to_string(),
    };

    let response = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&new_playlist)
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

async fn delete_playlist(playlist_id: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/playlist/{}", backend_url, playlist_id);

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

async fn force_check_complete_playlist(playlist_id: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/monitor/playlist/{}/check", backend_url, playlist_id);

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

async fn toggle_playlist_active(playlist_id: &str, active: bool) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!(
        "{}/monitor/playlist/{}/{}",
        backend_url,
        playlist_id,
        if active { "activate" } else { "deactivate" }
    );

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

async fn toggle_channel_active(channel_id: &str, active: bool) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!(
        "{}/monitor/channel/{}/{}",
        backend_url,
        channel_id,
        if active { "activate" } else { "deactivate" }
    );

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
