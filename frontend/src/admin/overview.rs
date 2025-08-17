use crate::admin::api::{load_admin_stats, login_admin};
use crate::admin::components::{AdminLayout, Dashboard, ErrorMessage, LoginForm};
use crate::admin::models::AdminStats;
use crate::admin::utils::{get_stored_admin_token, remove_admin_token, store_admin_token};
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub mod admin_captions;
pub mod admin_monitor;
pub mod admin_queue;
pub mod admin_videos;

#[derive(Properties, PartialEq)]
pub struct AdminPageProps {}

#[function_component(AdminPage)]
pub fn admin_page(_props: &AdminPageProps) -> Html {
    let admin_token = use_state(get_stored_admin_token);
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
                            let _ = store_admin_token(&token);
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
            let _ = remove_admin_token();
            admin_token.set(None);
            is_authenticated.set(false);
            stats.set(None);
            login_token_input.set(String::new());
            error_message.set(None);
        })
    };

    html! {
        <AdminLayout title="Admin Panel">
            <ErrorMessage error_message={(*error_message).clone()} />

            {
                if *is_authenticated {
                    html! {
                        <Dashboard
                            stats={(*stats).clone().unwrap_or_default()}
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
        </AdminLayout>
    }
}
