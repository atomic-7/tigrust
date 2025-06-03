use crate::meteo::{Options, Query, Timezone};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct HourlyData {
    // this seems to cause proplems while deserializing, despite the docs
    // saying that DateTime has support for deserializing from ISO8601 via RFC339 parsing
    // https://docs.rs/chrono/latest/chrono/serde/index.html
    // 2025-03-24T20:00
    // time: Vec<chrono::DateTime<Utc>>,
    pub temperature_2m: Vec<f32>,
    pub precipitation: Vec<f32>,
    pub weathercode: Vec<u32>,
}

#[derive(Clone, Deserialize)]
pub struct DailyData {
    pub weathercode: Vec<u32>,
    pub precipitation_sum: Vec<f32>,
    pub sunshine_duration: Vec<f32>,
    pub temperature_2m_max: Vec<f32>,
    pub temperature_2m_min: Vec<f32>,
}

#[derive(Clone, Deserialize)]
pub struct QueryResponse {
    pub hourly: HourlyData,
    pub daily: DailyData,
}

pub fn build_query() -> Query {
    Query::new()
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
        ]))
}
