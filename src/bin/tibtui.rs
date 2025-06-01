use color_eyre::Result;
use ratatui::crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};
use serde::Deserialize;
use tigrust::meteo::{MeteoAPI, Options, Query, Timezone};
// use ratatui::widgets::{BarChart, BarGroup};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result.await
}

#[derive(Deserialize)]
struct HourlyData {
    // this seems to cause proplems while deserializing, despite the docs
    // saying that DateTime has support for deserializing from ISO8601 via RFC339 parsing
    // https://docs.rs/chrono/latest/chrono/serde/index.html
    // 2025-03-24T20:00
    // time: Vec<chrono::DateTime<Utc>>,
    temperature_2m: Vec<f32>,
    precipitation: Vec<f32>,
    weathercode: Vec<u32>,
}

#[derive(Deserialize)]
struct DailyData {
    weathercode: Vec<u32>,
    precipitation_sum: Vec<f32>,
    sunshine_duration: Vec<f32>,
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
}

#[derive(Deserialize)]
struct QueryResponse {
    hourly: HourlyData,
    daily: DailyData,
}

async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let api = MeteoAPI::new();
    let query = Query::new()
        .with(Options::LatLong(48.165318, 11.754293))
        .with(Options::Timezone(Timezone::Auto))
        .with(Options::ForecastDays(3))
        .with(Options::Hourly(vec![
            String::from("temperature_2m"),
            String::from("precipitation"),
            String::from("weathercode"),
        ]))
        .with(Options::Daily(vec![
            String::from("weathercode"),
            String::from("precipitation_sum"),
            String::from("sunshine_duration"),
            String::from("temperature_2m_max"),
            String::from("temperature_2m_min"),
        ]));
    let _response = match api.do_query::<QueryResponse>(&query).await {
        Err(e) => {
            eprintln!("failed to get weather info: {:?}", e);
            panic!("something went wrong");
        }
        Ok(res) => res,
    };
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("hello world", frame.area());
}
