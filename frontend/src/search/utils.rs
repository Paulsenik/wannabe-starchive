use crate::models::FilterParameter;
use web_sys::console;

pub fn get_query_param() -> Option<String> {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|search| web_sys::UrlSearchParams::new_with_str(&search).ok())
        .and_then(|params| {
            let result = params.get("q");
            match &result {
                Some(val) => console::log_1(&format!("query-param: {}", val).into()),
                None => console::log_1(&"query-param: Not found".into()),
            }
            result
        })
}

pub fn get_filter_param() -> Option<FilterParameter> {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|search| web_sys::UrlSearchParams::new_with_str(&search).ok())
        .and_then(|params| {
            let mut search_type = "natural".to_string();
            match &params.get("t") {
                Some(val) => {
                    search_type = match val.as_str() {
                        "natural" => "natural".to_string(),
                        "wide" => "wide".to_string(),
                        _ => "natural".to_string(),
                    };
                    console::log_1(&format!("search-type: {}", search_type).into());
                }
                None => console::log_1(&"search-type: Not found".into()),
            }
            Some(FilterParameter { search_type })
        })
}
