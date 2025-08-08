use crate::models::VideoMetadata;
use crate::router::Route;
use crate::{format_iso8601_date, format_iso8601_duration, format_number};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AdminVideosPageProps {}

#[function_component(AdminVideosPage)]
pub fn admin_videos_page(_props: &AdminVideosPageProps) -> Html {
    let videos = use_state(|| Vec::<VideoMetadata>::new());
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let current_page = use_state(|| 1);
    let total_items = use_state(|| 0);
    let per_page = use_state(|| 10);

    // Clone states for pagination
    let current_page_display = current_page.clone();
    let per_page_display = per_page.clone();
    let total_items_display = total_items.clone();

    // Load videos on component mount
    {
        let videos = videos.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let total_items = total_items.clone();

        use_effect_with(*current_page, move |_| {
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match load_videos(*current_page, *per_page).await {
                    Ok(response) => {
                        videos.set(response.videos);
                        total_items.set(response.total);
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
                        let updated_videos: Vec<VideoMetadata> = current_videos
                            .into_iter()
                            .filter(|v| v.video_id != video_id)
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
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Upload Date"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Duration"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Views"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Likes"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Comments"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Captions"}</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">{"Actions"}</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white divide-y divide-gray-200">
                                            {
                                                (*videos).iter().map(|video| {
                                                    let video_id = video.video_id.clone();
                                                    let on_delete = on_delete_video.clone();
                                                    let channel_link = format!("https://www.youtube.com/channel/{}", &video.channel_id);

                                                    html! {
                                                        <tr key={video.video_id.clone()}>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                <div class="max-w-xs truncate"><a href={format!("https://www.youtube.com/watch?v={}", video.video_id)} class="text-blue-600 hover:underline">{&video.title}</a></div>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                <a href={format!("https://www.youtube.com/channel/{}",&video.channel_id)} class="text-blue-600 hover:underline">{&video.channel_name}</a>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {format_iso8601_date(&video.upload_date)}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {format_iso8601_duration(&video.duration)}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {format_number(video.views)}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {format_number(video.likes)}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {format_number(video.comment_count)}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                                {if video.has_captions { "Yes" } else { "No" }}
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
                                    <div class="mt-4 flex justify-between items-center">
                                        <div class="text-sm text-gray-700">
                                            {format!("Showing {} to {} of {} results",
                                                ((*current_page_display - 1) * *per_page_display + 1),
                                                (*current_page_display * *per_page_display).min(*total_items_display),
                                                *total_items_display
                                            )}
                                        </div>
                                        <div class="flex space-x-2">
                                            <button
                                                onclick={
                                                    let current_page = current_page_display.clone();
                                                    Callback::from(move |_| {
                                                        if *current_page > 1 {
                                                            current_page.set(*current_page - 1);
                                                        }
                                                    })
                                                }
                                                disabled={*current_page_display <= 1}
                                                class="px-3 py-2 border rounded-md disabled:opacity-50"
                                            >
                                                {"Previous"}
                                            </button>
                                            <button
                                                onclick={
                                                    let current_page = current_page_display.clone();
                                                    let per_page = per_page_display.clone();
                                                    let total_items = total_items_display.clone();
                                                    Callback::from(move |_| {
                                                        if (*current_page * *per_page) < *total_items {
                                                            current_page.set(*current_page + 1);
                                                        }
                                                    })
                                                }
                                                disabled={(*current_page_display * *per_page_display) >= *total_items}
                                                class="px-3 py-2 border rounded-md disabled:opacity-50"
                                            >
                                                {"Next"}
                                            </button>
                                        </div>
                                    </div>
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
struct VideosResponse {
    videos: Vec<VideoMetadata>,
    total: i64,
    page: i64,
    per_page: i64,
}

async fn load_videos(page: i64, per_page: i64) -> Result<VideosResponse, String> {
    let backend_url = "http://localhost:8000";
    let url = format!(
        "{}/admin/videos?page={}&per_page={}",
        backend_url, page, per_page
    );

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
            .json::<VideosResponse>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn delete_video(video_id: &str) -> Result<(), String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/video/{}", backend_url, video_id);

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
