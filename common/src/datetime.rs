#[derive(Clone, Copy)]
pub struct DateTime {
    pub timestamp: i64,
    pub offset: i32,
}

impl DateTime {
    /// Return the Unix timestamp and timezone offset in seconds.
    ///
    pub fn now() -> Self {
        cfg_if::cfg_if! { if #[cfg(feature = "ssr")] {
            use std::time::{SystemTime, UNIX_EPOCH};

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            Self {
                timestamp,
                offset: 0,
            }
        } else if #[cfg(feature = "hydrate")] {
            use web_sys::js_sys;

            let date = js_sys::Date::new_0();
            let timestamp = (date.get_time() / 1000.0) as i64;
            let offset = (date.get_timezone_offset() * 60.0) as i32;

            Self { timestamp, offset }
        } else {
            Self {
                timestamp: 0,
                offset: 0,
            }
        }}
    }

    pub fn fmt_sm(&self, ts: i64) -> String {
        if ts == 0 {
            return String::new();
        }

        let (days, extra) = self.get_local_date(ts);

        if self.is_same_day(ts) {
            let (h, m) = hms_from_seconds(extra);
            format!("{:02}:{:02}", h, m)
        } else if self.is_same_year(ts) {
            let (_, mo, d) = ymd_from_days(days);
            format!("{} {}", month_name(mo, true), d)
        } else {
            let (yr, ..) = ymd_from_days(days);
            format!("{}", yr)
        }
    }

    pub fn fmt_lg(&self, ts: i64) -> String {
        if ts == 0 {
            return String::new();
        }

        let (days, extra) = self.get_local_date(ts);

        if self.is_same_day(ts) {
            let (h, m) = hms_from_seconds(extra);
            format!("Today, {:02}:{:02}", h, m)
        } else if self.is_same_year(ts) {
            let (_, mo, d) = ymd_from_days(days);
            let (h, m) = hms_from_seconds(extra);
            format!("{} {}, {:02}:{:02}", month_name(mo, true), d, h, m)
        } else {
            let (yr, mo, d) = ymd_from_days(days);
            let (h, m) = hms_from_seconds(extra);
            format!("{} {}, {} {:02}:{:02}", month_name(mo, true), d, yr, h, m)
        }
    }

    /// Returns days from UNIX_EPOCH and extra seconds
    ///
    fn get_local_date(&self, ts: i64) -> (i32, i32) {
        let total = ts - self.offset as i64;
        let days = total / 86400;
        let extra = total - days * 86400;

        (days as i32, extra as i32)
    }

    /// Returns whether is the same day
    ///
    fn is_same_day(&self, ts: i64) -> bool {
        (self.timestamp / 86400) == (ts / 86400)
    }

    /// Returns whether is the same year
    ///
    fn is_same_year(&self, ts: i64) -> bool {
        (self.timestamp / 31536000) == (ts / 31536000)
    }
}

/// Returns year/month/day triple
///
/// days is number of days since UNIX_EPOCH
fn ymd_from_days(days: i32) -> (i32, i32, i32) {
    // shift from 1970-01-01 to 0000-03-01
    let days = days + 719468;

    // compute the era (days in an ear is 146097)
    let era = if days >= 0 {
        days / 146097
    } else {
        (days - 146096) / 146097
    };

    // compute the day of era (doe)
    let doe = days - era * 146097;

    // compute the year of era [0-399]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;

    // get the year
    let year = yoe + era * 400;

    // compute the day of year
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);

    let mp = (5 * doy + 2) / 153;

    // get the day of month
    let day = doy - (153 * mp + 2) / 5 + 1;

    // get the month
    let month = if mp < 10 { mp + 3 } else { mp - 9 };

    (year, month, day)
}

/// Returns hour/minute/second triple
///
/// extra is number of seconds since begin of today
fn hms_from_seconds(extra: i32) -> (i32, i32) {
    let hour = extra / 3600;

    let extra = extra - hour * 3600;
    let minute = extra / 60;

    (hour, minute)
}

/// Returns name of month
///
fn month_name(month: i32, sm: bool) -> String {
    match month {
        1 => {
            if sm {
                String::from("Jan")
            } else {
                String::from("January")
            }
        }
        2 => {
            if sm {
                String::from("Feb")
            } else {
                String::from("February")
            }
        }
        3 => {
            if sm {
                String::from("Mar")
            } else {
                String::from("March")
            }
        }
        4 => {
            if sm {
                String::from("Apr")
            } else {
                String::from("April")
            }
        }
        5 => String::from("May"),
        6 => String::from("June"),
        7 => String::from("July"),
        8 => {
            if sm {
                String::from("Aug")
            } else {
                String::from("August")
            }
        }
        9 => {
            if sm {
                String::from("Sept")
            } else {
                String::from("September")
            }
        }
        10 => {
            if sm {
                String::from("Oct")
            } else {
                String::from("October")
            }
        }
        11 => {
            if sm {
                String::from("Nov")
            } else {
                String::from("November")
            }
        }
        12 => {
            if sm {
                String::from("Dec")
            } else {
                String::from("December")
            }
        }
        _ => String::from("Error"),
    }
}
