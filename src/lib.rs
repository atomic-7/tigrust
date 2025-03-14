use rocket::request::FromParam;
use std::fmt::{self, Display};
pub mod cache;
pub mod tibber_api;

#[derive(Debug)]
pub enum Days {
    TODAY,
    TOMORROW,
}

impl Display for Days {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TODAY => write!(f, "today"),
            Self::TOMORROW => write!(f, "tomorrow"),
        }
    }
}

impl<'a> FromParam<'a> for Days {
    type Error = &'a str;
    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match param {
            "today" => Ok(Self::TODAY),
            "tomorrow" => Ok(Self::TOMORROW),
            _ => Err(param),
        }
    }
}
