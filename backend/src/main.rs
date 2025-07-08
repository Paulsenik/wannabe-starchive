#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/test")]
fn test() -> &'static str {
    "Test - Hello, world!"
}

#[get("/create/<username>")]
fn user_create(username: &str) -> String {
    format!("New User - Username: {username}")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, test])
        .mount("/user", routes![user_create])
}
