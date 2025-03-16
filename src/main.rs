use chrono;
use dotenv;
use rocket::State;
use rocket::serde::json::Json;
use rocket::fs::{relative, FileServer};
use rocket::tokio::time::{Duration, sleep};
use rocket::{get, put, routes};
use rocket_dyn_templates::{Template, context};
use std::str::from_utf8;
use tigrust::Days;
use tigrust::cache::{Cache, Error as CacheError};
use tigrust::tibber_api::{Prices, TibberAPI};
use tigrust::plot::Graph;


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

// fn graph_json(day: Days) -> Option<content::RawJson<String>> {
// need to figure out how to route by accept header
#[get("/prices/<day>", format = "application/json")]
async fn prices(
    day: Days,
    tapi: &State<TibberAPI<'_>>,
    cache: &State<Cache<Prices>>,
) -> Option<Json<Vec<f32>>> {
    if let Some(p) = prices_by_day(day, tapi, cache).await {
        return Some(Json(p))
    }
    None
}

async fn prices_by_day(day: Days, tapi: &State<TibberAPI<'_>>, cache: &State<Cache<Prices>>) -> Option<Vec<f32>> {

    let value_guard = cache.get_value();
    match value_guard {
        Ok(cached) => match cached.get(day) {
            Some(prices) => Some(prices.to_owned()),
            None => None,
        },
        Err(CacheError::CacheExpired) => {
            // drop the guard so it does not need to live across async boundaries
            drop(value_guard);
            match tapi.get_prices().await {
                Err(e) => {
                    eprintln!("failed to get prices: {}", e);
                    None
                }
                Ok(prices) => {
                    println!("hitting the api!");
                    // could probably be avoided by implementing DeRefMut, but I don't understand the
                    // implications yet
                    cache.update_value(prices.clone());
                    if let Some(data) = prices.get(day) {
                        Some(data.to_owned())
                    } else {
                        // no price data available for day, TODO: improve err msg
                        None
                    }
                }
            }
        }
    }
}

#[get("/graph/<day>")]
async fn graph(day: Days, tapi: &State<TibberAPI<'_>>, cache: &State<Cache<Prices>>) -> Template {
    let plot = match prices_by_day(day, tapi, cache).await {
        None => String::from(format!("prices for {} not available", day)),
        Some(prices) => {
            let mut buf = Vec::new();
            let graph = Graph::new();
            let _ = graph.plot(Box::new(&mut buf), &prices);
            from_utf8(buf.as_slice()).unwrap().to_string()
        }
    };
    Template::render("graph", context!{ graph: plot, refresh_rate: 60})
}

#[get("/time")]
fn current_time() -> String {
    format!("{}", chrono::offset::Local::now().format("%H:%M"))
}

#[get("/")]
async fn index() -> Template {
    Template::render("index", context!{time: current_time()})
}

// rocket::main launches the tokio runtime, if this is not desired leave it out and start the
// chosen runtime
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = dotenv::dotenv();
    let key = dotenv::var("TOKEN").expect("TOKEN needs to be placed in .env");
    // TODO: See if TibberAPI needs to do anything to be threadsafe
    let tapi = TibberAPI::new(key);
    let prices = tapi.get_prices().await.unwrap();
    let cache: Cache<Prices> = Cache::new(prices, 36000);
    // cache.debug_expire_cache();

    let _rocket = rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![index, delay, graph, current_time, store])
        .mount("/api/", routes![prices])
        .manage(tapi)
        .manage(cache)
        .mount("/static", FileServer::from(relative!("static")))
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
