use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

#[derive(Debug, Clone)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug, Clone)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}

#[derive(Debug, Clone)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(conflict) = s.parse::<ReservationConflict>() {
            Ok(ReservationConflictInfo::Parsed(conflict))
        } else {
            Ok(ReservationConflictInfo::Unparsed(s.to_string()))
        }
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<ParsedInfo>()?.try_into()
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();
    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            old: value.old.try_into()?,
        })
    }
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();
    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");
        let mut split = timespan_str.splitn(2, ',');
        let start = parse_pg_str_to_datetime(split.next().ok_or(())?.trim()).map_err(|_| ())?;
        let end = parse_pg_str_to_datetime(split.next().ok_or(())?.trim()).map_err(|_| ())?;
        let rid = value.get("resource_id").ok_or(())?.to_string();
        Ok(Self { rid, start, end })
    }
}

struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r#"\((?P<k1>[a-zA-Z0-9_-]+)\s*,\s*(?P<k2>[a-zA-Z0-9_-]+)\)=\((?P<v1>[a-zA-Z0-9_-]+)\s*,\s*\[(?P<v2>[^\)\]]+)"#).unwrap();
        let mut maps = vec![];
        for cap in re.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(
                cap.name("k1").unwrap().as_str().to_string(),
                cap.name("v1").unwrap().as_str().to_string(),
            );
            map.insert(
                cap.name("k2").unwrap().as_str().to_string(),
                cap.name("v2").unwrap().as_str().to_string(),
            );
            maps.push(Some(map));
        }

        if maps.len() != 2 {
            return Err(());
        }
        Ok(ParsedInfo {
            new: maps[0].take().unwrap(),
            old: maps[1].take().unwrap(),
        })
    }
}

// Parse pgsql datetime string into chrono DateTime<Utc>
fn parse_pg_str_to_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .to_utc())
}

#[cfg(test)]
mod tests {
    use super::*;
    const ERR_MSG: &str = "Key (resource_id, timespan)=(room-114514, [\"2025-06-02 19:00:00+00\", \"2025-06-05 19:00:00+00\"]) conflects with existing key (resource_id, timespan)=(room-114514, [\"2025-06-01 19:00:00+00\", \"2025-06-03 19:00:00+00\"])).";

    #[test]
    fn parsed_info_should_work() {
        let parsed_info: ParsedInfo = ERR_MSG.parse().unwrap();
        assert_eq!(parsed_info.new.get("resource_id").unwrap(), "room-114514");
        assert_eq!(
            parsed_info.new.get("timespan").unwrap(),
            "\"2025-06-02 19:00:00+00\", \"2025-06-05 19:00:00+00\""
        );
        assert_eq!(parsed_info.old.get("resource_id").unwrap(), "room-114514");
        assert_eq!(
            parsed_info.old.get("timespan").unwrap(),
            "\"2025-06-01 19:00:00+00\", \"2025-06-03 19:00:00+00\""
        );
    }

    #[test]
    fn parse_pg_str_to_datetime_should_work() {
        let datetime_str = "2025-06-02 19:00:00+00";
        let datetime = parse_pg_str_to_datetime(datetime_str).unwrap();
        assert_eq!(datetime.to_string(), "2025-06-02 19:00:00 UTC");
    }

    #[test]
    fn hashmap_to_reservation_window_should_work() {
        let parsed_info: ParsedInfo = ERR_MSG.parse().unwrap();
        let new_window: ReservationWindow = parsed_info.new.try_into().unwrap();
        assert_eq!(new_window.rid, "room-114514");
        assert_eq!(new_window.start.to_string(), "2025-06-02 19:00:00 UTC");
        assert_eq!(new_window.end.to_string(), "2025-06-05 19:00:00 UTC");

        let old_window: ReservationWindow = parsed_info.old.try_into().unwrap();
        assert_eq!(old_window.rid, "room-114514");
        assert_eq!(old_window.start.to_string(), "2025-06-01 19:00:00 UTC");
        assert_eq!(old_window.end.to_string(), "2025-06-03 19:00:00 UTC");
    }

    #[test]
    fn conflict_error_message_should_parse() {
        let conflict = ERR_MSG.parse::<ReservationConflictInfo>().unwrap();
        match conflict {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.new.rid, "room-114514");
                assert_eq!(conflict.new.start.to_string(), "2025-06-02 19:00:00 UTC");
                assert_eq!(conflict.new.end.to_string(), "2025-06-05 19:00:00 UTC");
                assert_eq!(conflict.old.rid, "room-114514");
                assert_eq!(conflict.old.start.to_string(), "2025-06-01 19:00:00 UTC");
                assert_eq!(conflict.old.end.to_string(), "2025-06-03 19:00:00 UTC");
            }
            ReservationConflictInfo::Unparsed(_) => panic!("Expected parsed conflict info"),
        }
    }
}
