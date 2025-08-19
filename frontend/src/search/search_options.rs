use js_sys::Reflect;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::Event;
use yew::{function_component, html, Callback, Html, Properties};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    UploadDate,
    Duration,
    Views,
    Likes,
    CaptionMatches,
}

impl SortBy {
    pub fn display_name(&self) -> &'static str {
        match self {
            SortBy::Relevance => "Relevance",
            SortBy::UploadDate => "Upload date",
            SortBy::Duration => "Duration",
            SortBy::Views => "Views",
            SortBy::Likes => "Likes",
            SortBy::CaptionMatches => "Caption matches",
        }
    }

    pub fn all_variants() -> Vec<Self> {
        vec![
            SortBy::Relevance,
            SortBy::UploadDate,
            SortBy::Duration,
            SortBy::Views,
            SortBy::Likes,
            SortBy::CaptionMatches,
        ]
    }
}

// NOTE: Adjust these variants if your actual enum differs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn display_name(&self) -> &'static str {
        match self {
            SortOrder::Asc => "Ascending",
            SortOrder::Desc => "Descending",
        }
    }
}

// Keys used in <option value="..."> so we can reliably map back and forth.
fn sort_by_key(sb: &SortBy) -> &'static str {
    match sb {
        SortBy::Relevance => "relevance",
        SortBy::UploadDate => "upload_date",
        SortBy::Duration => "duration",
        SortBy::Views => "views",
        SortBy::Likes => "likes",
        SortBy::CaptionMatches => "caption_matches",
    }
}

fn sort_by_from_key(key: &str) -> Option<SortBy> {
    match key {
        "relevance" => Some(SortBy::Relevance),
        "upload_date" => Some(SortBy::UploadDate),
        "duration" => Some(SortBy::Duration),
        "views" => Some(SortBy::Views),
        "likes" => Some(SortBy::Likes),
        "caption_matches" => Some(SortBy::CaptionMatches),
        _ => None,
    }
}

fn sort_order_key(so: &SortOrder) -> &'static str {
    match so {
        SortOrder::Asc => "asc",
        SortOrder::Desc => "desc",
    }
}

fn sort_order_from_key(key: &str) -> Option<SortOrder> {
    match key {
        "asc" => Some(SortOrder::Asc),
        "desc" => Some(SortOrder::Desc),
        _ => None,
    }
}

// Helper to read "value" from any event target without HtmlSelectElement.
fn event_value(e: &Event) -> Option<String> {
    let target = e.target()?;
    let js_value = Reflect::get(target.as_ref(), &JsValue::from_str("value")).ok()?;
    js_value.as_string()
}

#[derive(Properties, PartialEq)]
pub struct SearchOptionsProps {
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub on_sort_by_change: Callback<SortBy>,
    pub on_sort_order_change: Callback<SortOrder>,
}

#[function_component(SearchOptionsDropdowns)]
pub fn search_options(props: &SearchOptionsProps) -> Html {
    // onChange for SortBy
    let on_sort_by_change_cb = props.on_sort_by_change.clone();
    let on_sort_by_change = Callback::from(move |e: Event| {
        if let Some(value) = event_value(&e) {
            if let Some(sb) = sort_by_from_key(&value) {
                on_sort_by_change_cb.emit(sb);
            }
        }
    });

    // onChange for SortOrder
    let on_sort_order_change_cb = props.on_sort_order_change.clone();
    let on_sort_order_change = Callback::from(move |e: Event| {
        if let Some(value) = event_value(&e) {
            if let Some(order) = sort_order_from_key(&value) {
                on_sort_order_change_cb.emit(order);
            }
        }
    });

    let current_sort_by_key = sort_by_key(&props.sort_by).to_string();
    let current_sort_order_key = sort_order_key(&props.sort_order).to_string();

    html! {
        <div class="search-options">
            <label class="search-option">
                { "Sort by" }
                <select value={current_sort_by_key} onchange={on_sort_by_change}>
                    {
                        for SortBy::all_variants().into_iter().map(|sb| {
                            let key = sort_by_key(&sb).to_string();
                            html! {
                                <option value={key.clone()} selected={sb == props.sort_by}>
                                    { sb.display_name() }
                                </option>
                            }
                        })
                    }
                </select>
            </label>

            <label class="search-option">
                { "Order" }
                <select value={current_sort_order_key} onchange={on_sort_order_change}>
                    <option value="asc" selected={props.sort_order == SortOrder::Asc}>
                        { SortOrder::Asc.display_name() }
                    </option>
                    <option value="desc" selected={props.sort_order == SortOrder::Desc}>
                        { SortOrder::Desc.display_name() }
                    </option>
                </select>
            </label>
        </div>
    }
}
