use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::cache::Cache;
use crate::meteo::MeteoAPI;
use crate::query::{QueryResponse, build_query};
use color_eyre::Result;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::text::Text;
use ratatui::widgets::{Bar, BarChart, BarGroup, Clear};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
// use russh::keys::ssh_key::rand_core::OsRng;
use russh::keys::ssh_key::PublicKey;
use russh::server::*;
use russh::{Channel, ChannelId, Pty};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

type SshTerminal = Terminal<CrosstermBackend<TerminalHandle>>;

// server stuff
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
}

// The terminal handle represents a terminal to ratatui
struct TerminalHandle {
    sender: UnboundedSender<Vec<u8>>,
    sink: Vec<u8>, // sink collects data to send
}

impl TerminalHandle {
    async fn start(handle: Handle, channel_id: ChannelId) -> Self {
        // Handle is part of russh::server and represents a session
        let (sender, mut receiver) = mpsc::unbounded_channel::<Vec<u8>>();
        tokio::spawn(async move {
            while let Some(data) = receiver.recv().await {
                let result = handle.data(channel_id, data.into()).await;
                if result.is_err() {
                    eprintln!("Failed to send data: {:?}", result);
                }
            }
        });
        Self {
            sender,
            sink: Vec::new(),
        }
    }
}

impl std::io::Write for TerminalHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.sink.extend_from_slice(buf); // cache in the sink before encryption for transport
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let result = self.sender.send(self.sink.clone());
        if result.is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                result.unwrap_err(),
            ));
        }

        self.sink.clear();
        Ok(())
    }
}

#[derive(Clone)]
pub struct AppServer {
    // save each clients app state seperately
    clients: Arc<Mutex<HashMap<usize, (SshTerminal, App)>>>,
    id: usize,
    weather_cache: Arc<Cache<QueryResponse>>,
}

impl AppServer {
    pub async fn new() -> Self {
        let api = MeteoAPI::new();
        let query = build_query();

        let response = match api.do_query::<QueryResponse>(&query).await {
            Err(e) => {
                eprintln!("failed to get weather info: {:?}", e);
                panic!("something went wrong");
            }
            Ok(res) => res,
        };
        let weather_cache = Arc::new(Cache::new(response, 36000));
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            id: 0,
            weather_cache,
        }
    }

    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        let clients = self.clients.clone();
        // this needs to be changed
        let weather_cache = self.weather_cache.clone();
        let api = MeteoAPI::new();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // TODO: see if this makes sense
                for (_, (terminal, app)) in clients.lock().await.iter_mut() {
                    app.counter += 1;

                    // render the app here like normal
                    match app.page {
                        Page::Weather => {
                            let weather = match weather_cache.get_or_hydrate(&api).await {
                                Err(_) => return,
                                Ok(cached) => cached,
                            };
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
                            terminal
                                .draw(|f: &mut Frame| {
                                    let area = f.area();
                                    f.render_widget(Clear, area);
                                    f.render_widget(Text::from(format!("{}", app.counter)), area);
                                })
                                .unwrap();
                        }
                    }
                }
            }
        });

        let home = std::env::var("HOME").expect("unable to find path to home");
        let home = Path::new(&home);
        let key_path = home.join(".ssh").join("tigrust-localhost");
        // Config is part of russh::server
        let config = Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
            auth_rejection_time: std::time::Duration::from_secs(3),
            auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
            keys: vec![
                russh::keys::PrivateKey::read_openssh_file(&key_path)
                    .expect("unable to open ssh key"),
            ],
            nodelay: true,
            ..Default::default() // part of russh::server
        };

        self.run_on_address(Arc::new(config), ("0.0.0.0", 2222))
            .await?;

        Ok(())
    }
}

impl Drop for AppServer {
    fn drop(&mut self) {
        let id = self.id;
        let clients = self.clients.clone();
        tokio::spawn(async move {
            let mut clients = clients.lock().await;
            clients.remove(&id);
        });
    }
}

impl Server for AppServer {
    type Handler = Self;
    // handler needs to satisfy static lifetime
    fn new_client(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        s
    }
}

impl Handler for AppServer {
    type Error = anyhow::Error;
    // incoming session
    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        session: &mut Session,
    ) -> Result<bool, Self::Error> {
        let terminal_handle = TerminalHandle::start(session.handle(), channel.id()).await;
        let backend = CrosstermBackend::new(terminal_handle);
        let options = TerminalOptions {
            viewport: Viewport::Fixed(Rect::default()),
        };
        let terminal = Terminal::with_options(backend, options)?;
        let app = App::new();
        let mut clients = self.clients.lock().await;
        clients.insert(self.id, (terminal, app));
        println!("accepted session");
        Ok(true)
    }

    // Just accept all incoming connections with all auth methods for now
    async fn auth_none(&mut self, user: &str) -> std::result::Result<Auth, Self::Error> {
        println!("got connection from {}", user);
        Ok(Auth::Accept)
    }
    async fn auth_password(&mut self, _: &str, _: &str) -> std::result::Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }
    async fn auth_publickey(&mut self, _: &str, pubkey: &PublicKey) -> Result<Auth, Self::Error> {
        dbg!(pubkey);
        Ok(Auth::Accept)
    }

    // update
    // incoming data, react to incoming events. maybe dispatch events to a queue that is handled in
    // the main loop?
    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        match data {
            b"q" => {
                self.clients.lock().await.remove(&self.id);
                session.close(channel)?;
            }
            b"n" => {
                let mut clients = self.clients.lock().await;
                let (_, app) = clients.get_mut(&self.id).unwrap();
                app.page = match app.page {
                    Page::Price => Page::Weather,
                    Page::Weather => Page::Price,
                }
            }
            _ => {}
        }
        Ok(())
    }

    // clients windows size changed
    async fn window_change_request(
        &mut self,
        _: ChannelId,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        let rect = Rect {
            x: 0,
            y: 0,
            width: col_width as u16,
            height: row_height as u16,
        };
        let mut clients = self.clients.lock().await;
        let (terminal, _) = clients.get_mut(&self.id).unwrap();
        terminal.resize(rect)?;
        Ok(())
    }

    // client requests pseudo terminal
    async fn pty_request(
        &mut self,
        channel: ChannelId,
        _: &str,
        col_width: u32,
        row_height: u32,
        _: u32,
        _: u32,
        _: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let rect = Rect {
            x: 0,
            y: 0,
            width: col_width as u16,
            height: row_height as u16,
        };
        let mut clients = self.clients.lock().await;
        let (terminal, _) = clients.get_mut(&self.id).unwrap(); // ignore app
        terminal.resize(rect)?;
        session.channel_success(channel)?;
        Ok(())
    }
}
