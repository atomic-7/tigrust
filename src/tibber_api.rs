use graphql_client::{GraphQLQuery, Response};
use http_body_util::BodyExt;
use hyper::{Request, body::Buf};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use std::{error::Error, io::Read};
use tokio::net::TcpStream;

const API_URL: &'static str = "https://api.tibber.com/v1-beta/gql";

#[derive(GraphQLQuery, Deserialize, Debug)]
#[graphql(
    schema_path = "schema/tibber_schema.graphql",
    query_path = "schema/price_query.graphql",
    response_derives = "Debug, Deserialize"
)]
pub struct GetPriceInfo;

pub struct TibberAPI {
    api_key: String,
}

impl TibberAPI {
    pub fn new(key: String) -> Self {
        Self { api_key: key }
    }
    pub async fn get_prices(&self) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
        let url: hyper::Uri = API_URL.parse().unwrap();
        let host = url.host().expect("uri has no host??");
        let port = url.port_u16().unwrap_or(443);
        let addr = format!("{}:{}", host, port);
        // TODO: use tokio-rustls to make this use https 
        let stream = TcpStream::connect(addr).await?;
        let io = TokioIo::new(stream);
        // stuck between these prints

        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection failed: {:?}", err);
            }
        });

        let variables = get_price_info::Variables {};
        let request_body = GetPriceInfo::build_query(variables);

        let authority = url.authority().unwrap().clone();
        let req = Request::builder()
            .uri(url)
            .method("POST")
            .header(hyper::header::HOST, authority.as_str())
            .header(
                hyper::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            )
            // TODO: figure out how to do this differently
            .body(request_body.query.to_string())?;
        let res = sender.send_request(req).await?;
        let body = res.collect().await?.aggregate();
        println!("body received");
        let mut content = String::new();
        body.reader().read_to_string(&mut content)?;
        dbg!(&content);
        // let prices: Response<GetPriceInfo> = serde_json::from_reader(body.reader())?;
        let serialized: Result<Response<GetPriceInfo>, serde_json::Error> =
            serde_json::from_str(&content);
        println!("deserialized prices");
        match serialized {
            Err(e) => {
                eprintln!(
                    "failed to deserialize body to Response<GetPriceInfo>: {:?}",
                    e
                );
                return Err(Box::new(e));
            }
            Ok(prices) => {
                dbg!(&prices);
            }
        }
        // let prices : Response<get_price_info::GetPriceInfoViewerHomesCurrentSubscriptionPriceInfo> = serde_json::from_reader(body.reader())?;
        Ok(Vec::new())
    }
}
