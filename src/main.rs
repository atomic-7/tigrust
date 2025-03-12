use chrono;
use dotenv;
use rocket::request::FromParam;
use rocket::serde::json::Json;
use rocket::tokio::time::{Duration, sleep};
use rocket::{get, put, routes};
use std::fmt::{self, Display};
use tigrust::tibber_api::TibberAPI;

// #[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

// route handlers can be async functions
// parameters are specified with <> brackets
#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Waited for {} seconds", seconds)
}

#[put("/store/<value>")]
fn store(value: i32) -> String {
    format!("stored your number({})!", value)
}

enum Days {
    TODAY,
    TOMORROW,
}

impl Display for Days {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TODAY => write!(f, "today"),
            Self::TOMORROW => write!(f, "tomorrow"),
        }
    }
}

impl<'a> FromParam<'a> for Days {
    type Error = &'a str;
    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match param {
            "today" => Ok(Self::TODAY),
            "tomorrow" => Ok(Self::TOMORROW),
            _ => Err(param),
        }
    }
}

// TODO: look into bringing in serde directly

// fn graph_json(day: Days) -> Option<content::RawJson<String>> {
// need to figure out how to route by accept header
#[get("/api/prices/<day>", format = "application/json")]
fn graph_json(day: Days) -> Option<Json<Vec<i32>>> {
    let prices = vec![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    ];
    let _ = dotenv::dotenv();
    
    let key = dotenv::var("TOKEN").expect("TOKEN needs to be placed in .env");
    let tapi = TibberAPI::new(key);
    match day {
        Days::TODAY => Some(Json(prices)),
        Days::TOMORROW => None,
    }
}

#[get("/graph/<day>")]
fn graph(day: Days) -> String {
    format!("graphing {}", day)
}

#[get("/time")]
fn current_time() -> String {
    format!("{}", chrono::offset::Local::now().format("%H:%M"))
}

// rocket::main launches the tokio runtime, if this is not desired leave it out and start the
// chosen runtime
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _tapi = TibberAPI::new();
    let _rocket = rocket::build()
        .mount(
            "/",
            routes![index, delay, graph_json, graph, current_time, store],
        )
        //.mount("/public", FileServer::from("/www/static"))
        .launch()
        .await?;
    Ok(())
}

// use rocket::{Rocket, Build};
// use rocket::launch;
// #[launch]
// fn rocket() -> Rocket<Build> {
//     // the / is the base path of the namespace
//     // doc example is "hello" as basepath, with world as the route -> /hello/world
//     // it is also possible to add a route to multiple basepaths
//     rocket::build().mount("/", routes![index])
// }
