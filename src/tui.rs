use crate::meteo::MeteoAPI;
use crate::ssh::SshTerminal;
use crate::tibber_api::TibberAPI;
use crate::{cache::HydratingCache, query::QueryResponse, tibber_api::Prices};
use ratatui::Frame;
use ratatui::text::Text;
use ratatui::widgets::{Bar, BarChart, BarGroup, Clear};

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
        weather_cache: &mut HydratingCache<QueryResponse, MeteoAPI<'static>>,
        price_cache: &mut HydratingCache<Prices, TibberAPI<'static>>,
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
                                .label(format!("B{}", i).into())
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
                let chart = BarChart::default().data(bars);
                terminal
                    .draw(|f: &mut Frame| {
                        let area = f.area();
                        f.render_widget(Clear, area);
                        f.render_widget(&chart, area);
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
                        let area = f.area();
                        f.render_widget(Clear, area);
                        f.render_widget(Text::from(format!("{}", self.counter)), area);
                        f.render_widget(chart, area);
                    })
                    .unwrap();
            }
        }
    }
}
