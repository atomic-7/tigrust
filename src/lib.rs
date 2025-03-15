use rocket::request::FromParam;
use std::fmt::{self, Display};
pub mod cache;
pub mod tibber_api;

// TODO: check where the copy semantics make life easier
#[derive(Copy,Clone, Debug)]
pub enum Days {
    TODAY,
    TOMORROW,
}

impl Days {
    pub fn other(&self) -> Self {
        match self {
            Self::TODAY => Self::TOMORROW,
            Self::TOMORROW => Self::TODAY,
        }
    }
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

#[derive(Clone, Debug)]
pub struct Prices {
    pub today: Option<Vec<f32>>,
    pub tomorrow: Option<Vec<f32>>,
}

impl Prices {
    pub fn new() -> Self {
        Self {
            today: None,
            tomorrow: None,
        }
    }
    pub fn get(&self, day: Days) -> &Option<Vec<f32>> {
        match day {
            Days::TODAY => &self.today,
            Days::TOMORROW => &self.tomorrow,
        }
    }
    pub fn update(&mut self, day: Days, prices: Vec<f32>) {
        match day {
            Days::TODAY => self.today = Some(prices),
            Days::TOMORROW => self.tomorrow = Some(prices),
        }
    }
}
