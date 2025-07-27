use crate::router::Route;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::{window, HtmlInputElement};
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AdminStats {
    pub total_videos: i64,
    pub total_captions: i64,
    pub last_crawl_time: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct AdminPageProps {}

#[derive(Properties, PartialEq)]
pub struct ErrorMessageProps {
    pub error_message: Option<String>,
}
fn format_iso8601_time_since(iso_date: &str) -> String {
    if iso_date == "Never" {
        return String::from("Never");
    }

    let now = chrono::Utc::now();
    let date = match chrono::DateTime::parse_from_rfc3339(iso_date) {
        Ok(d) => d.with_timezone(&chrono::Utc),
        Err(_) => return String::from("Invalid date"),
    };

    let duration = now.signed_duration_since(date);
    let seconds = duration.num_seconds();

    if seconds < 60 {
        return format!("{}s", seconds);
    }

    let minutes = seconds / 60;
    if minutes < 60 {
        let remaining_seconds = seconds % 60;
        return format!("{}m {}s ago", minutes, remaining_seconds);
    }

    let hours = minutes / 60;
    if hours < 24 {
        let remaining_minutes = minutes % 60;
        return format!("{}h {}m ago", hours, remaining_minutes);
    }

    let days = hours / 24;
    let remaining_hours = hours % 24;
    format!("{}d {}h ago", days, remaining_hours)
}

#[function_component(ErrorMessage)]
pub fn error_message(props: &ErrorMessageProps) -> Html {
    if let Some(msg) = &props.error_message {
        html! {
            <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                { msg }
            </div>
        }
    } else {
        html! {}
    }
}

#[derive(Properties, PartialEq)]
pub struct StatsPanelProps {
    pub stats: Option<AdminStats>,
}

#[function_component(StatsPanel)]
pub fn stats_panel(props: &StatsPanelProps) -> Html {
    if let Some(stats_data) = &props.stats {
        html! {
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                <div class="bg-blue-100 p-4 rounded-lg">
                    <h3 class="text-lg font-semibold text-blue-800">{"Total Videos"}</h3>
                    <p class="text-2xl font-bold text-blue-600">{stats_data.total_videos}</p>
                </div>
                <div class="bg-green-100 p-4 rounded-lg">
                    <h3 class="text-lg font-semibold text-green-800">{"Total Captions"}</h3>
                    <p class="text-2xl font-bold text-green-600">{stats_data.total_captions}</p>
                </div>
                <div class="bg-purple-100 p-4 rounded-lg">
                    <h3 class="text-lg font-semibold text-purple-800">{"Last Crawl"}</h3>
                    <p class="text-2xl font-bold text-purple-600">
                        {format_iso8601_time_since(stats_data.last_crawl_time.as_deref().unwrap_or("Never"))}
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

#[derive(Properties, PartialEq)]
pub struct LoginFormProps {
    pub login_token_input: String,
    pub loading: bool,
    pub on_token_input: Callback<InputEvent>,
    pub on_login_submit: Callback<web_sys::SubmitEvent>,
}

#[function_component(LoginForm)]
pub fn login_form(props: &LoginFormProps) -> Html {
    html! {
        <form onsubmit={props.on_login_submit.clone()} class="max-w-md mx-auto">
            <div class="mb-4">
                <label class="block text-gray-700 text-sm font-bold mb-2">
                    {"Admin Token"}
                </label>
                <input
                    type="password"
                    class="w-full p-3 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    placeholder="Enter your admin token..."
                    value={props.login_token_input.clone()}
                    oninput={props.on_token_input.clone()}
                    disabled={props.loading}
                />
            </div>
            <button
                type="submit"
                disabled={props.loading}
                class="w-full bg-blue-600 text-white p-3 rounded hover:bg-blue-700 disabled:opacity-50"
            >
                {if props.loading { "Logging in..." } else { "Login" }}
            </button>
        </form>
    }
}

#[derive(Properties, PartialEq)]
pub struct DashboardProps {
    pub stats: AdminStats,
    pub loading: bool,
    pub on_logout: Callback<MouseEvent>,
}

#[function_component(Dashboard)]
pub fn dashboard(props: &DashboardProps) -> Html {
    html! {
        <div>
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-2xl font-semibold text-gray-800">{"Dashboard"}</h2>
                <button
                    onclick={props.on_logout.clone()}
                    class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
                >
                    {"Logout"}
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                <Link<Route> to={Route::AdminVideos} classes="bg-blue-600 text-white p-4 rounded text-center hover:bg-blue-700">
                    <div class="font-semibold text-lg mb-2">{"Manage Videos"}</div>
                    <div class="text-3xl font-bold">{props.stats.total_videos}</div>
                    <div class="text-sm opacity-80">{"Total Videos"}</div>
                </Link<Route>>
                <Link<Route> to={Route::AdminCaptions} classes="bg-green-600 text-white p-4 rounded text-center hover:bg-green-700">
                    <div class="font-semibold text-lg mb-2">{"Manage Captions"}</div>
                    <div class="text-3xl font-bold">{props.stats.total_captions}</div>
                    <div class="text-sm opacity-80">{"Total Captions"}</div>
                </Link<Route>>
                <Link<Route> to={Route::AdminQueue} classes="bg-purple-600 text-white p-4 rounded text-center hover:bg-purple-700">
                    <div class="font-semibold text-lg mb-2">{"Download Queue"}</div>
                    <div class="text-3xl font-bold">{format_iso8601_time_since(props.stats.last_crawl_time.as_deref().unwrap_or("Never"))}</div>
                    <div class="text-sm opacity-80">{"Last Crawl"}</div>
                </Link<Route>>
                <Link<Route> to={Route::AdminChannels} classes="bg-orange-600 text-white p-4 rounded text-center hover:bg-orange-700">
                    <div class="font-semibold text-lg mb-2">{"Manage Channels"}</div>
                    <div class="text-3xl font-bold">{"..."}</div>
                    <div class="text-sm opacity-80">{"Channel Management"}</div>
                </Link<Route>>
            </div>
        </div>
    }
}

#[function_component(AdminPage)]
pub fn admin_page(_props: &AdminPageProps) -> Html {
    let admin_token = use_state(|| {
        window()
            .and_then(|w| w.session_storage().ok())
            .and_then(|s| s.and_then(|storage| storage.get_item("admin_token").ok()))
            .flatten()
    });
    let login_token_input = use_state(|| String::new());
    let is_authenticated = use_state(|| admin_token.is_some());
    let loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let stats = use_state(|| None::<AdminStats>);

    // Load stats on component mount if already authenticated
    {
        let admin_token = admin_token.clone();
        let stats = stats.clone();
        let error_message = error_message.clone();

        use_effect_with((), move |_| {
            if let Some(token) = (*admin_token).clone() {
                let stats = stats.clone();
                let error_message = error_message.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    match load_admin_stats(&token).await {
                        Ok(stats_data) => {
                            stats.set(Some(stats_data));
                        }
                        Err(e) => {
                            error_message.set(Some(format!("Failed to load stats: {}", e)));
                        }
                    }
                });
            }
        });
    }

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
                            if let Some(window) = window() {
                                if let Ok(Some(storage)) = window.session_storage() {
                                    let _ = storage.set_item("admin_token", &token);
                                }
                            }
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

    let on_logout = {
        let admin_token = admin_token.clone();
        let is_authenticated = is_authenticated.clone();
        let stats = stats.clone();
        let login_token_input = login_token_input.clone();
        let error_message = error_message.clone();

        Callback::from(move |_| {
            if let Some(window) = window() {
                if let Ok(Some(storage)) = window.session_storage() {
                    let _ = storage.remove_item("admin_token");
                }
            }
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

                    <ErrorMessage error_message={(*error_message).clone()} />

                    {
                        if *is_authenticated {
                            html! {
                                <Dashboard
                                    stats={(*stats).clone().unwrap_or_else(|| AdminStats {
                                        total_videos: 0,
                                        total_captions: 0,
                                        last_crawl_time: None,
                                    })}
                                    loading={*loading}
                                    on_logout={on_logout}
                                />
                            }
                        } else {
                            html! {
                                <LoginForm
                                    login_token_input={(*login_token_input).clone()}
                                    loading={*loading}
                                    on_token_input={on_token_input}
                                    on_login_submit={on_login_submit}
                                />
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
