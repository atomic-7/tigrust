use chrono;
use dotenv;
use rocket::State;
use rocket::serde::json::Json;
use rocket::tokio::time::{Duration, sleep};
use rocket::{get, put, routes};
use tigrust::Days;
use tigrust::tibber_api::TibberAPI;

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

// TODO: look into bringing in serde directly

// fn graph_json(day: Days) -> Option<content::RawJson<String>> {
// need to figure out how to route by accept header
#[get("/prices/<day>", format = "application/json")]
async fn prices(day: Days, tapi: &State<TibberAPI<'_>>) -> Option<Json<Vec<f32>>> {
    match tapi.get_prices(day).await {
        Err(e) => {
            eprintln!("failed to get prices: {}", e);
            None
        }
        Ok(prices) => Some(Json(prices)),
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
    let _ = dotenv::dotenv();
    let key = dotenv::var("TOKEN").expect("TOKEN needs to be placed in .env");
    // TODO: See if TibberAPI needs to do anything to be threadsafe
    let tapi = TibberAPI::new(key);
    let _rocket = rocket::build()
        .mount("/", routes![index, delay, graph, current_time, store])
        .mount("/api/", routes![prices])
        .manage(tapi)
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
