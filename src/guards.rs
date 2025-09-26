use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
#[derive(Debug)]
pub enum UserAgent {
    Curl,
    Standard,
    Tolino,
}

#[derive(Debug)]
pub enum UserAgentError {
    NoUserAgent,
    UnknownUserAgent,
    UnsupportedUserAgent,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent {
    type Error = UserAgentError;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let agent = req.headers().get_one("user-agent");
        // dbg!(&agent);
        if let Some(ag) = agent {
            if ag.contains("tolino") {
                Outcome::Success(Self::Tolino)
            } else {
                Outcome::Success(Self::Standard)
            }
        } else {
            // maybe just serve standard for clients without user agent
            Outcome::Error((Status::BadRequest, UserAgentError::NoUserAgent))
        }
    }
}
