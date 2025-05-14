// use super::Tick;

// pub const TICKS_PER_MINUTE: Tick = 1;
pub type Minute = u8;
pub const MINUTES_PER_HOUR: Minute = 60; // 1 IRL second
pub type Hour = u8;
pub const HOURS_PER_DAY: Hour = 60; // 60 IRL seconds
pub type Day = u8;
pub const DAYS_PER_SEASON: Day = 10; // 10 IRL minutes

#[derive(Debug, PartialEq, Eq, Default)]
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

pub type Year = u32;

#[derive(Default)]
pub struct GameTime {
    pub minute: Minute,
    pub hour: Hour,
    pub day: Day,
    pub season: Season,
    pub year: Year,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GameTimeEvent {
    None,
    YearEnd,
}

impl GameTime {
    pub fn update(&mut self) -> GameTimeEvent {
        self.minute += 1;
        if self.minute < MINUTES_PER_HOUR {
            return GameTimeEvent::None;
        }
        self.hour += 1;
        if self.hour < HOURS_PER_DAY {
            return GameTimeEvent::None;
        }
        self.day += 1;
        if self.day < DAYS_PER_SEASON {
            return GameTimeEvent::None;
        }
        self.season = self.season.next();
        if self.season != Season::Spring {
            return GameTimeEvent::None;
        }
        self.year += 1;
        // TODO: return or emit event for end of year!
        // - gnomads (if implemented)
        // - you survivied a year message
        GameTimeEvent::YearEnd
    }
}
