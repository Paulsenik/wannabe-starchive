mod admin;
mod env_variable_utils;
mod models;
mod router;
mod search;
mod utils;

use crate::env_variable_utils::{get_api_base_url, get_app_name, is_debug_mode};
use crate::router::{switch, Route};
use web_sys::console;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();

    console::log_1(
        &format!(
            "NAME: \"{}\", API: \"{}\" DEBUG: \"{}\"",
            get_app_name(),
            get_api_base_url(),
            is_debug_mode()
        )
        .into(),
    );
}
