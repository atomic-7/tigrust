use std::error::Error;
use std::fmt::{self, Display};
use std::sync::Arc;

use async_trait::async_trait;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper::body::{Buf, Bytes};
use hyper_util::rt::TokioIo;
use serde::de::DeserializeOwned;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::ServerName;

use crate::cache::Hydrator;
use crate::query::{QueryResponse, build_query};

pub struct Forecast {
    pub precipation_probability: Vec<f32>,
}

const API: &'static str = "https://api.open-meteo.com/v1/forecast";

pub struct MeteoAPI {
    tls_connector: TlsConnector,
}

impl MeteoAPI {
    pub fn new() -> Self {
        let config = rustls_platform_verifier::tls_config();
        let connector = TlsConnector::from(Arc::new(config));
        Self {
            tls_connector: connector,
        }
    }

    pub async fn do_query<T: DeserializeOwned>(
        &self,
        query: &Query,
    ) -> Result<T, Box<dyn Error + Send + Sync>> {
        let url: hyper::Uri = API.parse().unwrap();
        let host = url.host().expect("uri has no host??");
        let port = url.port_u16().unwrap_or(443);
        let addr = format!("{}:{}", host, port);
        let dns_name = ServerName::try_from("api.open-meteo.com").expect("invalid DNS name");
        let stream = TcpStream::connect(addr).await?;
        let tls_stream = self
            .tls_connector
            .connect(dns_name.to_owned(), stream)
            .await?;
        let io = TokioIo::new(tls_stream);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        // This should start polling the connection
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection failed: {:?}", err);
            }
        });

        let query_url = format!("{}?{}", url.clone(), query);

        // necessary for host header in http11
        let authority = url.authority().unwrap().clone();
        let req = Request::builder()
            .uri(query_url)
            .method("GET")
            .header(hyper::header::HOST, authority.as_str())
            .body(Empty::<Bytes>::new())?;
        let res = sender.send_request(req).await?;
        let body = res.collect().await?.aggregate();
        // let mut weather_info = String::new();
        // body.reader().read_to_string(&mut weather_info)?;
        // println!("weather response: {}", weather_info);

        let result: T = serde_json::from_reader(body.reader())?;
        Ok(result)
    }
}

#[async_trait]
impl Hydrator<QueryResponse> for MeteoAPI {
    async fn hydrate(&self) -> Result<QueryResponse, crate::cache::Error> {
        let query = build_query();
        match self.do_query(&query).await {
            Err(_) => Err(crate::cache::Error::HydrateError),
            Ok(result) => Ok(result),
        }
    }
}

// TODO: add proper error handling
#[derive(Debug)]
pub struct Query {
    pub options: Vec<Options>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
        }
    }
    pub fn with(mut self, opt: Options) -> Self {
        self.options.push(opt);
        self
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.options.is_empty() {
            return Ok(());
        }
        write!(f, "{}", self.options.first().unwrap())?;
        for opt in self.options[1..].iter() {
            write!(f, "&{}", opt)?;
        }
        Ok(())
    }
}

// timezone auto
#[derive(Debug)]
pub enum Options {
    LatLong(f32, f32),
    Hourly(Vec<String>),
    Daily(Vec<String>),
    Current(Vec<String>),
    ForecastDays(u32),
    ForecastHours(u32),
    Timezone(Timezone),
}

impl Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LatLong(lat, long) => write!(f, "latitude={}&longitude={}", lat, long),
            Self::Hourly(vars) => write!(f, "hourly={}", vars.join(",")),
            Self::Daily(vars) => write!(f, "daily={}", vars.join(",")),
            Self::Current(vars) => write!(f, "current={}", vars.join(",")),
            Self::ForecastDays(num) => write!(f, "forecast_days={}", num),
            Self::ForecastHours(num) => write!(f, "forecast_hours={}", num),
            Self::Timezone(tz) => write!(f, "timezone={}", tz),
        }
    }
}

#[derive(Debug)]
pub enum Timezone {
    Offset(i32),
    Auto,
}

impl Display for Timezone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Offset(offset) => write!(f, "{}", offset), // this might be incorrect
            Self::Auto => write!(f, "auto"),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn create_query() {
        let query = Query::new()
            .with(Options::LatLong(20., 30.))
            .with(Options::Hourly(vec![
                String::from("temperature_2m"),
                String::from("precipation_probability"),
                String::from("sunshine_duration"),
            ]));
        println!("{}", query);
        assert_eq!(
            "&latitude=20&longitude30&hourly=temperature_2m,precipation_probability,sunshine_duration",
            query.to_string()
        );
    }
}
