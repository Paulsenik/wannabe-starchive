use lazy_static::lazy_static;
use web_sys::window;

lazy_static! {
    pub static ref BACKEND_URL: String = get_backend_url();
}

pub fn get_env_var(key: &str) -> Option<String> {
    let window = window().expect("should have a window in this context");

    // Get the ENV_CONFIG object
    let env_config = js_sys::Reflect::get(&window, &"ENV_CONFIG".into()).ok()?;

    // Check if env_config is undefined
    if env_config.is_undefined() {
        log::warn!("ENV_CONFIG is undefined - environment variables not loaded");
        return None;
    }

    // Get the specific environment variable
    let value = js_sys::Reflect::get(&env_config, &key.into()).ok()?;

    // Convert to string if it's not undefined
    if !value.is_undefined() {
        value.as_string()
    } else {
        log::warn!("Environment variable '{}' is undefined", key);
        None
    }
}

pub fn get_backend_url() -> String {
    get_env_var("BACKEND_URL").unwrap_or_else(|| "http://localhost:8000".to_string())
}

pub fn get_app_name() -> String {
    get_env_var("APP_NAME").unwrap_or_else(|| "Paulsenik's StarCitizen Content Search".to_string())
}

pub fn is_debug_mode() -> bool {
    get_env_var("DEBUG_MODE")
        .unwrap_or_else(|| "false".to_string())
        .parse()
        .unwrap_or(false)
}
