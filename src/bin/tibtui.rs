use color_eyre::Result;
use ratatui::crossterm::event::{self, Event};
use ratatui::widgets::BarChart;
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
struct QueryResponse {
    hourly: HourlyData,
}

async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let api = MeteoAPI::new();
    let query = Query::new()
        .with(Options::LatLong(48.165318, 11.754293))
        .with(Options::Timezone(Timezone::Auto))
        .with(Options::ForecastDays(1))
        .with(Options::Hourly(vec![
            String::from("temperature_2m"),
            String::from("precipitation"),
            String::from("weathercode"),
        ]));
    let response = match api.do_query::<QueryResponse>(&query).await {
        Err(e) => {
            eprintln!("failed to get weather info: {:?}", e);
            panic!("something went wrong");
        }
        Ok(res) => res,
    };
    let idxs = (0..response.hourly.temperature_2m.len()).map(|idx| format!("B{:?}", idx));
    let data = response
        .hourly
        .temperature_2m
        .iter()
        .zip(idxs)
        .map(|(&t, i)| (i, t as u64))
        .collect::<Vec<_>>();
    let barchart = BarChart::default().data(&data);
    loop {
        terminal.draw(|frame: &mut Frame| {
            frame.render_widget(&barchart, frame.area());
            // frame.render_widget("hello world", frame.area());
        })?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}
