use chrono::{self, Duration, Timelike};
use dotenv;
use rocket::State;
use rocket::fs::{FileServer, relative};
use rocket::serde::json::Json;
use rocket::{get, routes};
use rocket_dyn_templates::{Template, context};
use serde::Serialize;
use std::str::from_utf8;
use tigrust::cache::{Cache, Error as CacheError};
use tigrust::meteo::MeteoAPI;
use tigrust::plot::prices::Graph;
use tigrust::query::{QueryResponse, build_query};
use tigrust::tibber_api::{Days, Prices, TibberAPI};
use tigrust::weathercodes::WeatherCodes;

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

async fn weather_info(
    cache: &State<Cache<QueryResponse>>,
    weather_api: &State<MeteoAPI<'_>>,
) -> Result<QueryResponse, String> {
    println!("serving weather from cache!");
    let value_guard = cache.get_value();
    match value_guard {
        Ok(cached) => Ok(cached.clone()),
        Err(CacheError::CacheExpired) => {
            drop(value_guard);
            let query = build_query();
            match weather_api.do_query(&query).await {
                Ok(res) => Ok(res),
                Err(e) => {
                    eprintln!("failed to get weather: {}", e);
                    Err(format!("weather_api: {}", e))
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

#[derive(Serialize)]
struct DaySection {
    icon: &'static str,
    temperature: f32,
    rain_sum: f32,
}

#[derive(Clone, Serialize)]
struct FutureDay {
    icon: &'static str,
    min_temp: f32,
    max_temp: f32,
    sun_hours: f32,
    rain_sum: f32,
}

#[get("/weather")]
async fn weather(
    cache: &State<Cache<QueryResponse>>,
    weather_api: &State<MeteoAPI<'_>>,
) -> Template {
    let result = match weather_info(cache, weather_api).await {
        Ok(r) => r,
        Err(err) => {
            return Template::render("errmsg", context! { error_msg: err });
        }
    };
    let temperatures_avg: Vec<f32> = result.hourly.temperature_2m[0..24]
        .chunks(3)
        .map(|c| (c[0] + c[1] + c[2]) / 3.0)
        .collect();
    let rain_sums: Vec<f32> = result.hourly.precipitation[0..24]
        .chunks(3)
        .map(|p| p[0] + p[1] + p[2])
        .collect();
    // TODO: Figure out when the day rolls over for the weather
    // use weather codes to decide icons
    // right now it only uses the first hour of everyt tripple
    // TODO: Figure out how to aggregate weathercodes
    let icons: Vec<&'static str> = result.hourly.weathercode[0..24]
        .chunks(3)
        .map(|codes| WeatherCodes::from_num(codes[0]).to_icon())
        .collect();
    let mut day_sections = Vec::new();
    for idx in 0..rain_sums.len() {
        day_sections.push(DaySection {
            icon: icons[idx],
            temperature: temperatures_avg[idx],
            rain_sum: rain_sums[idx],
        });
    }
    let mut forecasts: Vec<FutureDay> = Vec::new();
    for idx in 0..3 {
        forecasts.push(FutureDay {
            icon: WeatherCodes::from_num(result.daily.weathercode[idx]).to_icon(),
            min_temp: result.daily.temperature_2m_min[idx],
            max_temp: result.daily.temperature_2m_max[idx],
            sun_hours: result.daily.sunshine_duration[idx] / 3600.,
            rain_sum: result.daily.precipitation_sum[idx],
        });
    }
    Template::render(
        "weather",
        context! { refresh_rate: 3600, day_sections: day_sections, days: forecasts },
    )
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
    let price_cache: Cache<Prices> = Cache::new(prices, 36000);
    // cache.debug_expire_cache();

    let query = build_query();
    let weather_api = MeteoAPI::new();
    let weather_info: QueryResponse = weather_api.do_query(&query).await.unwrap();
    let weather_cache = Cache::new(weather_info, 36000);
    drop(query);

    let rocket = rocket::build()
        .attach(Template::fairing())
        .mount(
            "/",
            routes![index, current_time, current_price, graph, weather],
        )
        .mount("/api/", routes![prices])
        .manage(tapi)
        .manage(price_cache)
        .manage(weather_api)
        .manage(weather_cache)
        .mount("/static", FileServer::from(relative!("static")));

    // use tokio::select! with a join handle from spawning the ssh serv
    rocket.launch().await?;
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
