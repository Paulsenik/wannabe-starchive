use crate::router::Route;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub channel_name: String,
    pub duration: Option<String>,
    pub upload_date: Option<String>,
    pub view_count: Option<i64>,
}

#[derive(Properties, PartialEq)]
pub struct AdminVideosPageProps {}

#[function_component(AdminVideosPage)]
pub fn admin_videos_page(_props: &AdminVideosPageProps) -> Html {
    let videos = use_state(Vec::<Video>::new);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);

    // Load videos on component mount
    {
        let videos = videos.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();

        use_effect_with((), move |_| {
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match load_videos().await {
                    Ok(video_list) => {
                        videos.set(video_list);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to load videos: {}", e)));
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let on_delete_video = {
        let videos = videos.clone();
        let error_message = error_message.clone();

        Callback::from(move |video_id: String| {
            let videos = videos.clone();
            let error_message = error_message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match delete_video(&video_id).await {
                    Ok(_) => {
                        // Remove video from list
                        let current_videos = (*videos).clone();
                        let updated_videos: Vec<Video> = current_videos
                            .into_iter()
                            .filter(|v| v.id != video_id)
                            .collect();
                        videos.set(updated_videos);
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to delete video: {}", e)));
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
                            {"Videos"}
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
                                    <p>{"Loading videos..."}</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="overflow-x-auto">
                                    <table class="min-w-full bg-white border border-gray-300">
                                        <thead class="bg-gray-50">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Title"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Channel"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Duration"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Views"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Actions"}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {
                                                (*videos).iter().map(|video| {
                                                    let video_id = video.id.clone();
                                                    let on_delete = on_delete_video.clone();

                                                    html! {
                                                        <tr key={video.id.clone()}>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                <div class="max-w-xs truncate">{&video.title}</div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {&video.channel_name}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {video.duration.as_deref().unwrap_or("N/A")}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {video.view_count.map(|v| v.to_string()).unwrap_or_else(|| "N/A".to_string())}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                                                                <button
                                                                    onclick={
                                                                        let video_id = video_id.clone();
                                                                        let on_delete = on_delete.clone();
                                                                        Callback::from(move |_| {
                                                                            on_delete.emit(video_id.clone());
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

async fn load_videos() -> Result<Vec<Video>, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/videos", backend_url);

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
            .json::<Vec<Video>>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn delete_video(video_id: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/videos/{}", backend_url, video_id);

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
