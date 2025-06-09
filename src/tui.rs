use crate::meteo::MeteoAPI;
use crate::ssh::SshTerminal;
use crate::tibber_api::TibberAPI;
use crate::{cache::HydratingCache, query::QueryResponse, tibber_api::Prices};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Text;
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Clear};
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

                terminal
                    .draw(|f: &mut Frame| {
                        f.render_widget(Clear, f.area());
                        render_chart(f, bars, "Weather");
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

                terminal
                    .draw(|f: &mut Frame| {
                        // TODO: dedupe this
                        f.render_widget(Clear, f.area());
                        render_chart(f, bars, "Prices");
                    })
                    .unwrap();
            }
        }
    }
}

fn render_chart(f: &mut Frame, bars: BarGroup, title: &str) {
    let chart = BarChart::default().data(bars);
    let total_area = f.area();
    let bar_gap = if total_area.width > 140 { 2 } else { 1 };
    let bar_width = (total_area.width / 24) - bar_gap;
    let chart_width: u16 = 24 * (bar_width + bar_gap) - bar_gap;
    let mut chart = chart.bar_width(bar_width).bar_gap(bar_gap);
    if total_area.width < 96 {
        chart = chart.direction(ratatui::layout::Direction::Horizontal);
    }

    let chart = chart.bar_width(bar_width).bar_gap(bar_gap);
    let area = center(
        total_area,
        Constraint::Length(chart_width),
        Constraint::Percentage(95),
    );
    let [area, keybind_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Max(1)]).areas(area);
    let block = Block::bordered().title(format!("Tigrust - {} ", title));
    f.render_widget(&block, area);
    f.render_widget(
        chart,
        block.inner(center(area, Constraint::Fill(1), Constraint::Fill(1))),
    );
    f.render_widget(
        Text::from(format!("{} | Switch Page: n", title)),
        keybind_area,
    );
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

#[allow(unused)]
fn debug_info(frame: &mut Frame) {
    let area = frame.area();
    frame.render_widget(
        Text::from(format!("Area {}x{}", area.width, area.height)),
        area,
    );
}
