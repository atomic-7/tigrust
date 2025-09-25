use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
#[derive(Debug)]
pub enum UserAgent {
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
        dbg!(&agent);
        if let Some(ag) = agent {
            if ag.contains("Firefox") {
                Outcome::Success(Self::Standard)
            } else {
                Outcome::Error((Status::BadRequest, UserAgentError::UnknownUserAgent))
            }
        } else {
            Outcome::Error((Status::BadRequest, UserAgentError::NoUserAgent))
        }
    }
}
