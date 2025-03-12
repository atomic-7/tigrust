use graphql_client::GraphQLQuery;
use hyper_util::rt::TokioIo;
use std::error::Error;
use hyper::{body::Buf, Request, Response};
use http_body_util::BodyExt;
use tokio::net::TcpStream;
// use serde::Deserialize;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema/tibber_schema.graphql",
    query_path = "schema/price_query.graphql",
    response_derives = "Debug, Deserialize"
)]
pub struct GetPriceInfo;

pub struct TibberAPI {
    value: i32,
}

impl TibberAPI {
    pub fn new() -> Self {
        Self { value: 42 }
    }
}

const API_URL: &'static str = "https://api.tibber.com/v1-beta/gql";
pub async fn get_prices() -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
    let url: hyper::Uri = API_URL.parse().unwrap();
    let host = url.host().expect("uri has no host??");
    let port = url.port_u16().unwrap_or(80);
    let addr = format!("{}:{}", host, port);

    let stream = TcpStream::connect(addr).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    // wait until the handshake is done
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
        .header(hyper::header::AUTHORIZATION, format!("Bearer {}", "tood-put-token-here"))
        // TODO: figure out how to do this differently
        .body(request_body.query.to_string())?;
    let res = sender.send_request(req).await?;
    let body = res.collect().await?.aggregate();
    let prices: Response<GetPriceInfo> = serde_json::from_reader(body.reader())?;
    // let prices : Response<get_price_info::GetPriceInfoViewerHomesCurrentSubscriptionPriceInfo> = serde_json::from_reader(body.reader())?;

    Ok(Vec::new())
}
