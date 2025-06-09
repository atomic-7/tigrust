use crate::meteo::MeteoAPI;
use crate::ssh::SshTerminal;
use crate::tibber_api::TibberAPI;
use crate::{cache::HydratingCache, query::QueryResponse, tibber_api::Prices};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;
use ratatui::text::Text;
use ratatui::widgets::{Bar, BarChart, BarGroup, Clear};
use std::sync::Arc;

pub enum Page {
    Weather,
    Price,
}
pub struct App {
    pub counter: usize,
    pub page: Page,
}

impl App {
    pub fn new() -> App {
        // TODO: fetch weather data and price data here
        // TODO: later setup link between server data and ssh tui
        Self {
            counter: 0,
            page: Page::Weather,
        }
    }

    pub async fn render(
        &mut self,
        terminal: &mut SshTerminal,
        weather_cache: Arc<HydratingCache<QueryResponse, MeteoAPI>>,
        price_cache: Arc<HydratingCache<Prices, TibberAPI>>,
    ) {
        match self.page {
            Page::Weather => {
                let weather = weather_cache.get().await.unwrap();
                let bars = BarGroup::default().bars(
                    weather
                        .hourly
                        .temperature_2m
                        .iter()
                        .take(24)
                        .enumerate()
                        .map(|(i, &t)| {
                            Bar::default()
                                .value(t as u64)
                                .label(format!("{}", i).into())
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                let chart = BarChart::default().data(bars);
                terminal
                    .draw(|f: &mut Frame| {
                        let total_area = f.area();
                        let bar_width = if total_area.width < 48 { 2 } else { 4 }; // TODO: adjust depending on space
                        let bar_gap = 2;
                        let chart_width: u16 = 24 * (bar_width + bar_gap);
                        let chart = chart.bar_width(bar_width).bar_gap(bar_gap);
                        
                        let area = center(total_area, Constraint::Length(chart_width), Constraint::Percentage(95));
                        f.render_widget(Clear, area);
                        f.render_widget(&chart, area);
                        f.render_widget(Text::from(format!("Weather | Counter {}", self.counter)), area);
                    })
                    .unwrap();
            }
            Page::Price => {
                let prices = price_cache.get().await.unwrap();
                let bars = BarGroup::default().bars(
                    prices
                        .today
                        .as_ref()
                        .unwrap()
                        .iter()
                        .enumerate()
                        .map(|(i, &p)| {
                            Bar::default()
                                .value((p * 100.0) as u64)
                                .label(format!("{}", i).into())
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                let chart = BarChart::default().data(bars);
                terminal
                    .draw(|f: &mut Frame| {
                        // TODO: dedupe this
                        let total_area = f.area();
                        let bar_width = if total_area.width < 48 { 2 } else { 4 }; // TODO: adjust depending on space
                        let bar_gap = 2;
                        let chart_width: u16 = 24 * (bar_width + bar_gap);
                        let chart = chart.bar_width(bar_width).bar_gap(bar_gap);
                        let area = center(total_area, Constraint::Length(chart_width), Constraint::Percentage(95));
                        f.render_widget(Clear, area);
                        f.render_widget(chart, area);
                        f.render_widget(Text::from(format!("Prices | Counter {}", self.counter)), area);
                    })
                    .unwrap();
            }
        }
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
   let [area] = Layout::horizontal([horizontal])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    area
}
