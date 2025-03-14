// use get_price_info::GetPriceInfoViewerHomesCurrentSubscriptionPriceInfoToday;
use graphql_client::{GraphQLQuery, Response};
// use graphql_client::GraphQLQuery;
use http_body_util::BodyExt;
use hyper::{Request, body::Buf};
use hyper_util::rt::TokioIo;
// use serde::Deserialize;
use std::sync::Arc;
use std::{error::Error, io::Read};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::ServerName;
// use crate::price_query::get_price_info::GetPriceInfoViewer;
use crate::price_query::GetPriceInfo;
use crate::price_query;

// #[derive(GraphQLQuery, Deserialize, Debug)]
// #[derive(GraphQLQuery, Deserialize, Debug)]
// #[graphql(
//     schema_path = "schema/tibber_schema.graphql",
//     query_path = "schema/price_query.graphql",
//     response_derives = "Debug, Deserialize, Serialize"
// )]
// pub struct GetPriceInfo;

type ApiResult<T> = Result<T, serde_json::Error>;

pub struct TibberAPI<'a> {
    api_key: String,
    dnsname: ServerName<'a>,
    tls_connector: TlsConnector,
}

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

    pub async fn get_prices(&self) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
        let url: hyper::Uri = API_URL.parse().unwrap();
        let host = url.host().expect("uri has no host??");
        let port = url.port_u16().unwrap_or(443);
        let addr = format!("{}:{}", host, port);
        dbg!(&addr);
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

        let variables = price_query::get_price_info::Variables {};
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
        println!("body received");
        let mut content = String::new();
        body.reader().read_to_string(&mut content)?;
        dbg!(&content);
        // let prices: Respons&content = "{\"data\":{\"viewer\":{\"homes\":[{\"currentSubscription\":{\"priceInfo\":{\"today\":[{\"total\":0.3334,\"e<GetPriceInfo> = serde_json::from_reader(body.reader())?;
        // let serialized : Result<Response<get_price_info::GetPriceInfoViewerHomesCurrentSubscriptionPriceInfo>, serde_json::Error> = serde_json::from_str(&content);
        let serialized: ApiResult<Response<GetPriceInfo>> =
            serde_json::from_str(&content);
        dbg!(&serialized);
        // let serialized: Result<Response<GetPriceInfo>, serde_json::Error> =
        //     serde_json::from_str(&content);
        println!("deserialized prices");
        if let Err(e) = serialized {
            eprintln!("failed to deserialize body {:?}", e);
            return Err(Box::new(e));
        }
        let response = serialized.unwrap();
        dbg!(&response);
        // if let None = &response.data {
        //     panic!("no prices");
        // }
        // let data = response.data.unwrap();
        // dbg!(&data);
        Ok(Vec::new())
    }
}
