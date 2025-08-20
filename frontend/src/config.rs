use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref BACKEND_URL: String =
        env::var("BACKEND_URL").unwrap_or("http://localhost:8000".parse().unwrap());
}
