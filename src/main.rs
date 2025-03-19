use chrono::{self, Duration, Local, Timelike, Utc};
use dotenv;
use rocket::State;
use rocket::fs::{FileServer, relative};
use rocket::serde::json::Json;
use rocket::{get, routes};
use rocket_dyn_templates::{Template, context};
use serde::Deserialize;
use std::str::from_utf8;
use tigrust::Days;
use tigrust::cache::{Cache, Error as CacheError};
use tigrust::meteo::{MeteoAPI, Query, Options};
use tigrust::plot::prices::Graph;
use tigrust::tibber_api::{Prices, TibberAPI};

#[get("/current_price", format = "text/html")]
async fn current_price(tapi: &State<TibberAPI<'_>>, cache: &State<Cache<Prices>>) -> Template {
    // TOOD: See if current_price template and current_time template could just be one
    let now = chrono::Local::now();
    let hour_delta = now.with_minute(0).unwrap().with_second(1).unwrap() + Duration::hours(1) - now;
    let prices = prices_by_day(Days::TODAY, tapi, cache).await.unwrap();
    Template::render(
        "current_price",
        context! {refresh_rate: hour_delta.num_seconds(), price: format!("{:.1} ct", prices[now.hour() as usize] * 100.0 ) },
    )
}

// need to figure out how to route by accept header
#[get("/prices/<day>", format = "application/json")]
async fn prices(
    day: Days,
    tapi: &State<TibberAPI<'_>>,
    cache: &State<Cache<Prices>>,
) -> Option<Json<Vec<f32>>> {
    if let Some(p) = prices_by_day(day, tapi, cache).await {
        return Some(Json(p));
    }
    None
}

async fn prices_by_day(
    day: Days,
    tapi: &State<TibberAPI<'_>>,
    cache: &State<Cache<Prices>>,
) -> Option<Vec<f32>> {
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
    let now = chrono::Local::now();
    let hour_delta = now.with_minute(0).unwrap().with_second(1).unwrap() + Duration::hours(1) - now;
    let plot = match prices_by_day(day, tapi, cache).await {
        None => String::from(format!("prices for {} not available", day)),
        Some(prices) => {
            let mut buf = Vec::new();
            let current_hour: usize = now.hour().try_into().unwrap();
            let graph = Graph::new(current_hour);
            let _ = graph.plot(Box::new(&mut buf), &prices);
            from_utf8(buf.as_slice()).unwrap().to_string()
        }
    };
    Template::render(
        "graph",
        context! { graph: plot, refresh_rate: hour_delta.num_seconds()},
    )
}

#[get("/time")]
fn current_time() -> Template {
    let now = chrono::Local::now();
    let minute_delta = now.with_second(0).unwrap() + Duration::minutes(1) - now;
    Template::render(
        "time",
        context! {time: format!("{}", chrono::offset::Local::now().format("%H:%M")), refresh_rate: minute_delta.num_seconds()},
    )
}

#[derive(Deserialize)]
struct HourlyData {
    // this seems to cause proplems while deserializing, despite the docs
    // saying that DateTime has support for deserializing from ISO8601 via RFC339 parsing
    // https://docs.rs/chrono/latest/chrono/serde/index.html
    // 2025-03-24T20:00
    // time: Vec<chrono::DateTime<Utc>>,    
    temperature_2m: Vec<f32>,
    precipitation_probability: Vec<f32>,
    sunshine_duration: Vec<f32>,
}

#[derive(Deserialize)]
struct QueryResponse {
   hourly: HourlyData, 
}

#[get("/weather")]
async fn weather(weather_api: &State<MeteoAPI<'_>>) -> Template {
    let query = Query::new()
        .with(Options::LatLong(48.165318, 11.754293))
        .with(Options::Hourly(vec![
            String::from("temperature_2m"),
            String::from("precipitation_probability"),
            String::from("sunshine_duration"),
        ]));
    println!("weather query: {}", &query);
    let result = match weather_api.do_query::<QueryResponse>(&query).await {
        Err(e) => {
            eprintln!("failed to get weather info {}", e);
            String::from("request failed")
        }
        Ok(res) => {
            String::from(format!("weathering now! {}", res.hourly.temperature_2m[0]))
        }
    };
    println!("{}", result);
    Template::render("weather", context!{ refresh_rate: 3600 })
}

#[get("/")]
async fn index() -> Template {
    let now = chrono::Local::now();
    let tomorrow_delta = now.with_hour(0).unwrap() + Duration::days(1) - now;
    Template::render(
        "index",
        context! {refresh_rate: tomorrow_delta.num_seconds()},
    )
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
    
    let weather_api = MeteoAPI::new();

    let _rocket = rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![current_time, current_price, graph, index])
        .mount("/api/", routes![prices, weather])
        .manage(tapi)
        .manage(cache)
        .manage(weather_api)
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
