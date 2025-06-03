use serde::Deserialize;

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct DailyData {
    pub weathercode: Vec<u32>,
    pub precipitation_sum: Vec<f32>,
    pub sunshine_duration: Vec<f32>,
    pub temperature_2m_max: Vec<f32>,
    pub temperature_2m_min: Vec<f32>,
}

#[derive(Deserialize)]
pub struct QueryResponse {
    pub hourly: HourlyData,
    pub daily: DailyData,
}
