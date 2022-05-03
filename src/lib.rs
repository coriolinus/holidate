use serde::{Deserialize, Deserializer};
use time::Date;

#[derive(Debug, parse_display::Display, Deserialize)]
pub enum HolidayType {
    Public,
    Bank,
    School,
    Authorities,
    Optional,
    Observance,
}

// The Nager API provides several other fields than these, but we don't care
// about them for this use case, and `serde_json` conveniently just ignores
// any fields which aren't present in the struct.
#[derive(Debug, Deserialize)]
pub struct Holiday {
    pub date: Date,
    pub name: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub counties: Vec<String>,
    pub types: Vec<HolidayType>,
}

pub fn next_holidays(
    country: &str,
    relative_to: Date,
    quantity: usize,
) -> Result<Vec<Holiday>, Error> {
    let mut year = relative_to.year();
    let mut holidays = Vec::new();
    while holidays.len() < quantity {
        let mut new_holidays: Vec<Holiday> = get_holidays_cached(year, country)?;
        new_holidays.retain(|holiday| holiday.date >= relative_to);
        holidays.extend(new_holidays);
        year += 1;
    }
    holidays.truncate(quantity);
    Ok(holidays)
}

fn uri_for(year: i32, country_code: &str) -> String {
    format!("https://date.nager.at/api/v3/publicholidays/{year}/{country_code}")
}

fn get_holidays_cached(year: i32, country: &str) -> Result<Vec<Holiday>, Error> {
    // TODO! check cache, fill cache, invalidate cache if too old, etc
    // for now just naively hit the API every time
    // TODO! intercept empty body error, produce UnknownCountry error
    reqwest::blocking::get(uri_for(year, country))?
        .error_for_status()?
        .json()
        .map_err(Into::into)
}

/// Helper for serde to deserialize a `null` value as its default value.
///
/// From https://github.com/serde-rs/serde/issues/1098#issuecomment-760711617
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown country code")]
    UnknownCountry,
    #[error("http problem")]
    Reqwest(#[from] reqwest::Error),
}
