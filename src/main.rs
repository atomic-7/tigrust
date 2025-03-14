use chrono;
use dotenv;
use rocket::State;
use rocket::serde::json::Json;
use rocket::tokio::time::{Duration, sleep};
use rocket::{get, put, routes};
use std::sync::RwLock;
use tigrust::Days;
use tigrust::cache::Cache;
use tigrust::tibber_api::TibberAPI;

#[derive(Clone, Debug)]
struct Prices {
    today: Option<Vec<f32>>,
    tomorrow: Option<Vec<f32>>,
}

impl Prices {
    pub fn get(&self, day: Days) -> &Option<Vec<f32>> {
        match day {
            Days::TODAY => &self.today,
            Days::TOMORROW => &self.tomorrow,
        }
    }
    pub fn update(&mut self, day: Days, prices: Vec<f32>) {
        match day {
            Days::TODAY => self.today = Some(prices),
            Days::TOMORROW => self.tomorrow = Some(prices),
        }
    }
}

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

// fn graph_json(day: Days) -> Option<content::RawJson<String>> {
// need to figure out how to route by accept header
#[get("/prices/<day>", format = "application/json")]
async fn prices(
    day: Days,
    tapi: &State<TibberAPI<'_>>,
    cache: &State<RwLock<Cache<Prices>>>,
) -> Option<Json<Vec<f32>>> {
    // TODO: see if the RW lock can be made part of the cache
    {   // This scope limits the lifetime of the read lease on the cache
        let read_cache = cache.read().unwrap();
        if read_cache.is_valid() {
            match read_cache.get_value().unwrap().get(day) {
                Some(prices) => return Some(Json(prices.to_owned())),
                None => return None,
            }
        }
    }
    match tapi.get_prices(&day).await {
        Err(e) => {
            eprintln!("failed to get prices: {}", e);
            None
        }
        Ok(prices) => {
            let mut write_cache = cache.write().unwrap();
            println!("hitting the api!");
            let mut invalid = write_cache.get_unchecked().clone();
            invalid.update(day, prices.clone());
            write_cache.update_value(invalid.clone());
            Some(Json(prices))
        }
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
    let mut prices = Prices {
        today: None,
        tomorrow: None,
    };
    match tapi.get_prices(&Days::TODAY).await {
        Err(e) => {
            eprintln!("failed to get todays prices on startup: {}", e);
        }
        Ok(todays_prices) => {
            prices.today = Some(todays_prices);
        }
    }
    match tapi.get_prices(&Days::TOMORROW).await {
        Err(e) => {
            prices.tomorrow = None;
            eprintln!("failed to get todays prices on startup: {}", e);
        }
        Ok(tomorrows_prices) => {
            prices.tomorrow = Some(tomorrows_prices);
        }
    }
    let cache: RwLock<Cache<Prices>> = RwLock::new(Cache::new(prices, 36000));
    {
        cache.write().unwrap().debug_invalidate_cache();
    }

    let _rocket = rocket::build()
        .mount("/", routes![index, delay, graph, current_time, store])
        .mount("/api/", routes![prices])
        .manage(tapi)
        .manage(cache)
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
