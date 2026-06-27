// use super::Tick;

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::Tick;

// pub const TICKS_PER_MINUTE: Tick = 1;
// pub type Minute = u8;
pub const TICKS_PER_HOUR: Tick = 120; // 2 IRL seconds
pub type Hour = u8;
pub const HOURS_PER_DAY: Hour = 24; // 48 IRL seconds
pub const fn hours(hours: Hour) -> Tick {
    hours as Tick * TICKS_PER_HOUR
}
pub type Day = u8;
pub const fn days(days: Day) -> Tick {
    (days as Hour * HOURS_PER_DAY) as Tick * TICKS_PER_HOUR
}
pub const DAYS_PER_SEASON: Day = 5; // ~ 5 IRL minutes

#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn next(&self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Fall,
            Season::Fall => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
}

#[derive(Debug)]
pub struct ParseSeasonError;

impl FromStr for Season {
    type Err = ParseSeasonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spring" => Ok(Season::Spring),
            "summer" => Ok(Season::Summer),
            "fall" => Ok(Season::Fall),
            "winter" => Ok(Season::Winter),
            _ => Err(ParseSeasonError),
        }
    }
}

pub type Year = u32;

#[derive(Default, Serialize, Deserialize)]
pub struct GameTime {
    pub tick_off: Tick,
    pub hour: Hour,
    pub day: Day,
    pub season: Season,
    pub year: Year,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum GameTimeEvent {
    YearEnd,
}

pub fn print_days_nice(days: usize) -> String {
    let seasons = days / DAYS_PER_SEASON as usize;
    let years = (seasons as usize) / 4;
    let seasons = seasons % 4;
    let days = days % DAYS_PER_SEASON as usize;
    if years > 1 {
        format!("{} years", years)
    } else if years > 0 {
        format!("{} year {} seasons", years, seasons)
    } else if seasons > 0 {
        format!("{} seasons {} days", seasons, days)
    } else {
        format!("{} days", days)
    }
}

impl GameTime {
    pub fn update(&mut self) -> Option<GameTimeEvent> {
        self.tick_off += 1;
        if self.tick_off < TICKS_PER_HOUR {
            return None;
        }
        self.tick_off = 0;
        self.hour += 1;
        if self.hour < HOURS_PER_DAY {
            return None;
        }
        self.hour = 0;
        self.day += 1;
        if self.day < DAYS_PER_SEASON {
            return None;
        }
        self.day = 0;
        self.season = self.season.next();
        if self.season != Season::Spring {
            return None;
        }
        self.year += 1;
        // TODO: return or emit event for end of year!
        // - gnomads (if implemented)
        // - you survivied a year message
        Some(GameTimeEvent::YearEnd)
    }

    pub(crate) fn season_start(&self, season: Season) -> bool {
        self.season == season && self.tick_off == 0 && self.hour == 0 && self.day == 0
    }
}
