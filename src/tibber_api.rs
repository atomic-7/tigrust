use crate::Days;
use graphql_client::{GraphQLQuery, Response};
use http_body_util::BodyExt;
use hyper::{Request, body::Buf};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use std::fmt::{self, Display};
use std::sync::Arc;
use std::{error::Error, io::Read};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::ServerName;

#[derive(GraphQLQuery, Deserialize, Debug)]
#[graphql(
    schema_path = "schema/tibber_schema.graphql",
    query_path = "schema/price_query.graphql",
    response_derives = "Debug, Deserialize, Serialize"
)]
pub struct GetPriceInfo;

type ApiResult<T> = Result<T, serde_json::Error>;

pub struct TibberAPI<'a> {
    api_key: String,
    dnsname: ServerName<'a>,
    tls_connector: TlsConnector,
}

// TODO: look into thiserror crate
// TODO: look into how to derive the trait that allows boxing the error automagically
#[derive(Debug)]
pub enum ApiError {
    MissingField(String),
    NoPriceAvailable(Days),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "missing field {}", field),
            Self::NoPriceAvailable(day) => write!(f, "no prices available for {}", day),
        }
    }
}

impl Error for ApiError {}

const API_URL: &'static str = "https://api.tibber.com/v1-beta/gql";

impl<'a> TibberAPI<'a> {
    pub fn new(key: String) -> Self {
        let config = rustls_platform_verifier::tls_config();
        let connector = TlsConnector::from(Arc::new(config));
        let dnsname =
            ServerName::try_from("api.tibber.com").expect("api url should always be valid");
        Self {
            api_key: key,
            dnsname,
            tls_connector: connector,
        }
    }

    pub async fn get_prices(&self, day: &Days) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
        let url: hyper::Uri = API_URL.parse().unwrap();
        let host = url.host().expect("uri has no host??");
        let port = url.port_u16().unwrap_or(443);
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(addr).await?;
        let tls_stream = self
            .tls_connector
            .connect(self.dnsname.to_owned(), stream)
            .await?;
        let io = TokioIo::new(tls_stream);

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection failed: {:?}", err);
            }
        });

        let variables = get_price_info::Variables {};
        let request_body = GetPriceInfo::build_query(variables);
        let request_body = serde_json::to_string(&request_body)?;

        let authority = url.authority().unwrap().clone();
        let req = Request::builder()
            .uri(url)
            .method("POST")
            .header(hyper::header::HOST, authority.as_str())
            .header(
                hyper::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            )
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(request_body)?;
        let res = sender.send_request(req).await?;
        let body = res.collect().await?.aggregate();
        let mut content = String::new();
        body.reader().read_to_string(&mut content)?;

        let serialized: ApiResult<Response<get_price_info::ResponseData>> =
            serde_json::from_str(&content);
        let Ok(response) = serialized else {
            return Err(Box::new(serialized.unwrap_err()));
        };
        let Some(data) = response.data else {
            return Err(Box::new(ApiError::MissingField(String::from("data"))));
        };
        let Some(home) = &data.viewer.homes[0] else {
            return Err(Box::new(ApiError::MissingField(String::from("home"))));
        };
        let Some(sub) = &home.current_subscription else {
            return Err(Box::new(ApiError::MissingField(String::from(
                "subscription",
            ))));
        };
        let Some(price_info) = &sub.price_info else {
            return Err(Box::new(ApiError::MissingField(String::from("price_info"))));
        };

        // TODO: Improve error handling here
        let prices: Vec<f32> = match day {
            Days::TODAY => price_info
                .today
                .iter()
                .map(|p| p.as_ref().unwrap().total.unwrap() as f32)
                .collect(),
            Days::TOMORROW => price_info
                .tomorrow
                .iter()
                .map(|p| p.as_ref().unwrap().total.unwrap() as f32)
                .collect(),
        };
        if prices.is_empty() {
            return Err(Box::new(ApiError::NoPriceAvailable(day.clone())));
        }
        Ok(prices)
    }
}
