use crate::router::Route;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminLoginRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminLoginResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminStats {
    pub total_videos: i64,
    pub total_transcripts: i64,
    pub last_crawl_time: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct AdminPageProps {}

#[function_component(AdminPage)]
pub fn admin_page(_props: &AdminPageProps) -> Html {
    let admin_token = use_state(|| None::<String>);
    let login_token_input = use_state(|| String::new());
    let is_authenticated = use_state(|| false);
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let stats = use_state(|| None::<AdminStats>);

    let on_token_input = {
        let login_token_input = login_token_input.clone();
        Callback::from(move |e: InputEvent| {
            let input_value = e.target_unchecked_into::<HtmlInputElement>().value();
            login_token_input.set(input_value);
        })
    };

    let on_login_submit = {
        let login_token_input = login_token_input.clone();
        let admin_token = admin_token.clone();
        let is_authenticated = is_authenticated.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();
        let stats = stats.clone();

        Callback::from(move |e: web_sys::SubmitEvent| {
            e.prevent_default();

            let token = (*login_token_input).clone();
            let admin_token = admin_token.clone();
            let is_authenticated = is_authenticated.clone();
            let loading = loading.clone();
            let error_message = error_message.clone();
            let stats = stats.clone();

            if token.is_empty() {
                error_message.set(Some("Please enter an admin token".to_string()));
                return;
            }

            loading.set(true);
            error_message.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match login_admin(&token).await {
                    Ok(response) => {
                        if response.success {
                            admin_token.set(Some(token.clone()));
                            is_authenticated.set(true);

                            // Load stats after successful login
                            match load_admin_stats(&token).await {
                                Ok(stats_data) => {
                                    stats.set(Some(stats_data));
                                }
                                Err(e) => {
                                    error_message.set(Some(format!("Failed to load stats: {}", e)));
                                }
                            }
                        } else {
                            error_message.set(Some(response.message));
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Login failed: {}", e)));
                    }
                }
                loading.set(false);
            });
        })
    };

    let on_trigger_crawl = {
        let admin_token = admin_token.clone();
        let loading = loading.clone();
        let error_message = error_message.clone();

        Callback::from(move |_| {
            if let Some(token) = &*admin_token {
                let token = token.clone();
                let loading = loading.clone();
                let error_message = error_message.clone();

                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    match trigger_crawl(&token).await {
                        Ok(response) => {
                            if !response.success {
                                error_message.set(Some(response.message));
                            } else {
                                error_message
                                    .set(Some("Crawl triggered successfully!".to_string()));
                            }
                        }
                        Err(e) => {
                            error_message.set(Some(format!("Crawl trigger failed: {}", e)));
                        }
                    }
                    loading.set(false);
                });
            } else {
                error_message.set(Some("No admin token available".to_string()));
            }
        })
    };

    let on_logout = {
        let admin_token = admin_token.clone();
        let is_authenticated = is_authenticated.clone();
        let stats = stats.clone();
        let login_token_input = login_token_input.clone();
        let error_message = error_message.clone();

        Callback::from(move |_| {
            admin_token.set(None);
            is_authenticated.set(false);
            stats.set(None);
            login_token_input.set(String::new());
            error_message.set(None);
        })
    };

    html! {
        <div class="min-h-screen bg-gray-700 p-4">
            <div class="max-w-4xl mx-auto">
                <div class="bg-white rounded-lg shadow-lg p-8">
                    <div class="flex justify-between items-center mb-6">
                        <h1 class="text-3xl font-bold text-gray-800">
                            {"Admin Panel"}
                        </h1>
                        <Link<Route> to={Route::Home} classes="text-blue-600 hover:underline">
                            {"← Back to Search"}
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
                        if *is_authenticated {
                            html! {
                                <div>
                                    <div class="flex justify-between items-center mb-6">
                                        <h2 class="text-2xl font-semibold text-gray-800">{"Dashboard"}</h2>
                                        <button
                                            onclick={on_logout}
                                            class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
                                        >
                                            {"Logout"}
                                        </button>
                                    </div>

                                    {
                                        if let Some(stats_data) = &*stats {
                                            html! {
                                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                                                    <div class="bg-blue-100 p-4 rounded-lg">
                                                        <h3 class="text-lg font-semibold text-blue-800">{"Total Videos"}</h3>
                                                        <p class="text-2xl font-bold text-blue-600">{stats_data.total_videos}</p>
                                                    </div>
                                                    <div class="bg-green-100 p-4 rounded-lg">
                                                        <h3 class="text-lg font-semibold text-green-800">{"Total Transcripts"}</h3>
                                                        <p class="text-2xl font-bold text-green-600">{stats_data.total_transcripts}</p>
                                                    </div>
                                                    <div class="bg-purple-100 p-4 rounded-lg">
                                                        <h3 class="text-lg font-semibold text-purple-800">{"Last Crawl"}</h3>
                                                        <p class="text-sm text-purple-600">
                                                            {stats_data.last_crawl_time.as_deref().unwrap_or("Never")}
                                                        </p>
                                                    </div>
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <div class="bg-gray-100 p-4 rounded-lg mb-6">
                                                    <p class="text-gray-600">{"Loading stats..."}</p>
                                                </div>
                                            }
                                        }
                                    }

                                    <div class="bg-gray-50 p-4 rounded-lg">
                                        <h3 class="text-lg font-semibold text-gray-800 mb-4">{"Actions"}</h3>
                                        <button
                                            onclick={on_trigger_crawl}
                                            disabled={*loading}
                                            class="bg-blue-600 text-white px-6 py-2 rounded hover:bg-blue-700 disabled:opacity-50"
                                        >
                                            {if *loading { "Processing..." } else { "Trigger Crawl" }}
                                        </button>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {
                                <form onsubmit={on_login_submit} class="max-w-md mx-auto">
                                    <div class="mb-4">
                                        <label class="block text-gray-700 text-sm font-bold mb-2">
                                            {"Admin Token"}
                                        </label>
                                        <input
                                            type="password"
                                            class="w-full p-3 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            placeholder="Enter your admin token..."
                                            value={(*login_token_input).clone()}
                                            oninput={on_token_input}
                                            disabled={*loading}
                                        />
                                    </div>
                                    <button
                                        type="submit"
                                        disabled={*loading}
                                        class="w-full bg-blue-600 text-white p-3 rounded hover:bg-blue-700 disabled:opacity-50"
                                    >
                                        {if *loading { "Logging in..." } else { "Login" }}
                                    </button>
                                </form>
                            }
                        }
                    }
                </div>
            </div>
        </div>
    }
}

async fn login_admin(token: &str) -> Result<AdminLoginResponse, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/login", backend_url);

    let request_body = AdminLoginRequest {
        token: token.to_string(),
    };

    let response = Request::post(&url)
        .json(&request_body)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<AdminLoginResponse>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn load_admin_stats(token: &str) -> Result<AdminStats, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/stats", backend_url);

    let response = Request::get(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<AdminStats>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

async fn trigger_crawl(token: &str) -> Result<AdminLoginResponse, String> {
    let backend_url = "http://localhost:8000";
    let url = format!("{}/admin/trigger-crawl", backend_url);

    let response = Request::post(&url)
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<AdminLoginResponse>()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}
