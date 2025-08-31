use crate::admin::models::AdminStats;
use crate::admin::utils::format_iso8601_time_since;
use crate::router::Route;
use crate::utils::format_number;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorMessageProps {
    pub error_message: Option<String>,
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
                    <div class="text-3xl font-bold">{format_number(props.stats.total_videos)}</div>
                    <div class="text-sm opacity-80">{"Total Videos"}</div>
                </Link<Route>>
                <Link<Route> to={Route::AdminCaptions} classes="bg-green-600 text-white p-4 rounded text-center hover:bg-green-700">
                    <div class="font-semibold text-lg mb-2">{"Manage Captions"}</div>
                    <div class="text-3xl font-bold">{format_number(props.stats.total_captions)}</div>
                    <div class="text-sm opacity-80">{"Total Captions"}</div>
                </Link<Route>>
                <Link<Route> to={Route::AdminQueue} classes="bg-purple-600 text-white p-4 rounded text-center hover:bg-purple-700">
                    <div class="font-semibold text-lg mb-2">{"Manage Queue"}</div>
                    {
                        if props.stats.queue_size > 0 {
                            html! {
                                <>
                                    <div class="text-3xl font-bold">{format_number(props.stats.queue_size as i64)}</div>
                                    <div class="text-sm opacity-80">{"Items in Queue"}</div>
                                </>
                            }
                        } else {
                            html! {
                                <>
                                    <div class="text-3xl font-bold">{format_iso8601_time_since(props.stats.last_crawl_time.as_deref().unwrap_or("Never"))}</div>
                                    <div class="text-sm opacity-80">{"Last Crawl"}</div>
                                </>
                            }
                        }
                    }
                </Link<Route>>
                <Link<Route> to={Route::AdminMonitors} classes="bg-orange-600 text-white p-4 rounded text-center hover:bg-orange-700">
                    <div class="font-semibold text-lg mb-2">{"Manage Monitors"}</div>
                    <div class="text-3xl font-bold">{props.stats.active_monitors}</div>
                    <div class="text-sm opacity-80">{"Active Channel & Playlist Monitors"}</div>
                </Link<Route>>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AdminLayoutProps {
    pub children: Children,
    pub title: String,
}

#[function_component(AdminLayout)]
pub fn admin_layout(props: &AdminLayoutProps) -> Html {
    html! {
        <div class="min-h-screen bg-gray-700 p-4">
            <div class="max-w-4xl mx-auto">
                <div class="bg-white rounded-lg shadow-lg p-8">
                    <div class="flex justify-between items-center mb-6">
                        <h1 class="text-3xl font-bold text-gray-800">
                            {&props.title}
                        </h1>
                        <Link<Route> to={Route::Home} classes="text-blue-600 hover:underline">
                            {"← Back to Search"}
                        </Link<Route>>
                    </div>
                    { for props.children.iter() }
                </div>
            </div>
        </div>
    }
}
