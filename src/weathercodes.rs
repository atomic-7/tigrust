use std::include_str;
pub enum WeatherCodes {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,
    Fog,
    DepositingRimeFog,
    DrizzleLight,
    DrizzleModerate,
    DrizzleHeavy,
    FreezingDrizzleLight,
    FreezingDrizzleHeavy,
    RainSlight,
    RainModerate,
    RainHeavy,
    FreezingRainLight,
    FreezingRainHeavy,
    SnowSlight,
    SnowModerate,
    SnowHeavy,
    SnowGrains,
    RainShowerSlight,
    RainShowerModerate,
    RainShowerViolent,
    SnowShowerSlight,
    SnowShowerHeavy,
    ThunderstormSlight,
    ThunderstormModerate,
    ThunderStormHailSlight,
    ThunderStormHailHeavy,
    UnkownCode,
}

impl WeatherCodes {
    pub fn from_num(code: u32) -> Self {
        match code {
            0 => Self::ClearSky,
            1 => Self::MainlyClear,
            2 => Self::PartlyCloudy,
            3 => Self::Overcast,
            45 => Self::Fog,
            48 => Self::DepositingRimeFog,
            51 => Self::DrizzleLight,
            53 => Self::DrizzleModerate,
            55 => Self::DrizzleHeavy,
            56 => Self::FreezingDrizzleLight,
            57 => Self::FreezingDrizzleHeavy,
            61 => Self::RainSlight,
            63 => Self::RainModerate,
            65 => Self::RainHeavy,
            66 => Self::FreezingRainLight,
            67 => Self::FreezingRainHeavy,
            71 => Self::SnowSlight,
            73 => Self::SnowModerate,
            75 => Self::SnowHeavy,
            77 => Self::SnowGrains,
            80 => Self::RainShowerSlight,
            81 => Self::RainShowerModerate,
            82 => Self::RainShowerViolent,
            85 => Self::SnowShowerSlight,
            86 => Self::SnowShowerHeavy,
            95 => Self::ThunderstormSlight,
            195 => Self::ThunderstormModerate, // not really parsable
            96 => Self::ThunderStormHailSlight,
            99 => Self::ThunderStormHailHeavy,
            _ => Self::UnkownCode,
        }
    }
    pub fn to_icon(&self) -> &'static str {
        match self {
            Self::ClearSky => include_str!("../assets/weather_icons/weather-clear.svg"),
            _ => include_str!("../assets/weather_icons/action-unavailable.svg"),
        }
    }
    //  TODO: make function to average weather codes?
}
