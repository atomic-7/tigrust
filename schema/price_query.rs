#![allow(clippy::all, warnings)]
pub struct GetPriceInfo;
pub mod get_price_info {
    #![allow(dead_code)]
    use std::result::Result;
    pub const OPERATION_NAME: &str = "GetPriceInfo";
    pub const QUERY : & str = "query GetPriceInfo {\n\tviewer {\n\t\thomes {\n\t\t\tcurrentSubscription {\n\t\t\t\tpriceInfo {\n\t\t\t\t\ttoday {\n\t\t\t\t\t\ttotal\n\t\t\t\t\t\tenergy\n\t\t\t\t\t\ttax\n\t\t\t\t\t\tstartsAt\n\t\t\t\t\t}\n\t\t\t\t\ttomorrow {\n\t\t\t\t\t\ttotal\n\t\t\t\t\t\tenergy\n\t\t\t\t\t\ttax\n\t\t\t\t\t\tstartsAt\n\t\t\t\t\t}\n\t\t\t\t}\n\t\t\t}\n\t\t}\n\t}\n}\n" ;
    use super::*;
    use serde::{Deserialize, Serialize};
    #[allow(dead_code)]
    type Boolean = bool;
    #[allow(dead_code)]
    type Float = f64;
    #[allow(dead_code)]
    type Int = i64;
    #[allow(dead_code)]
    type ID = String;
    #[derive(Serialize, Debug)]
    pub struct Variables;
    #[derive(Deserialize, Debug)]
    pub struct ResponseData {
        pub viewer: GetPriceInfoViewer,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPriceInfoViewer {
        pub homes: Vec<Option<GetPriceInfoViewerHomes>>,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPriceInfoViewerHomes {
        #[serde(rename = "currentSubscription")]
        pub current_subscription: Option<GetPriceInfoViewerHomesCurrentSubscription>,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPriceInfoViewerHomesCurrentSubscription {
        #[serde(rename = "priceInfo")]
        pub price_info: Option<GetPriceInfoViewerHomesCurrentSubscriptionPriceInfo>,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPriceInfoViewerHomesCurrentSubscriptionPriceInfo {
        pub today: Vec<Option<GetPriceInfoViewerHomesCurrentSubscriptionPriceInfoToday>>,
        pub tomorrow: Vec<Option<GetPriceInfoViewerHomesCurrentSubscriptionPriceInfoTomorrow>>,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPriceInfoViewerHomesCurrentSubscriptionPriceInfoToday {
        pub total: Option<Float>,
        pub energy: Option<Float>,
        pub tax: Option<Float>,
        #[serde(rename = "startsAt")]
        pub starts_at: Option<String>,
    }
    #[derive(Deserialize, Debug)]
    pub struct GetPriceInfoViewerHomesCurrentSubscriptionPriceInfoTomorrow {
        pub total: Option<Float>,
        pub energy: Option<Float>,
        pub tax: Option<Float>,
        #[serde(rename = "startsAt")]
        pub starts_at: Option<String>,
    }
}
impl graphql_client::GraphQLQuery for GetPriceInfo {
    type Variables = get_price_info::Variables;
    type ResponseData = get_price_info::ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: get_price_info::QUERY,
            operation_name: get_price_info::OPERATION_NAME,
        }
    }
}
