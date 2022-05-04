use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use time::{Date, Duration, OffsetDateTime};

/// How long a cached list of holidays is valid for, before hitting the API
/// again to check for updates.
const CACHE_FADEOUT: Duration = Duration::hours(24);

#[derive(Debug, parse_display::Display, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct Holiday {
    pub date: Date,
    pub name: String,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub counties: Vec<String>,
    pub types: Vec<HolidayType>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CachedHoliday {
    /// when this cached page was fetched, for fadeout
    fetched: OffsetDateTime,
    year: i32,
    /// note that this is only ever lowercase
    country_code: String,
    holidays: Vec<Holiday>,
}

impl CachedHoliday {
    fn path(year: i32, country_code: &str) -> Result<PathBuf, Error> {
        Ok(dirs::cache_dir()
            .ok_or(Error::NoCacheDir)?
            .join("holidate")
            .join(country_code)
            .join(format!("{year}.json")))
    }

    fn load(year: i32, country_code: &str) -> Option<Vec<Holiday>> {
        let file = std::fs::File::open(Self::path(year, country_code).ok()?).ok()?;
        let reader = std::io::BufReader::new(file);
        let cache: Self = serde_json::from_reader(reader).ok()?;

        if cache.year != year
            || cache.country_code != country_code
            || cache.fetched + CACHE_FADEOUT < OffsetDateTime::now_utc()
        {
            None
        } else {
            Some(cache.holidays)
        }
    }

    fn store(&self) -> Result<(), Error> {
        let path = Self::path(self.year, &self.country_code)?;
        let dir = path
            .parent()
            .expect("Self::path never returns root directory");
        std::fs::create_dir_all(dir)?;
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
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

fn get_holidays_cached(year: i32, country_code: &str) -> Result<Vec<Holiday>, Error> {
    // the cache only ever deals with lowercase country codes, so let's compute
    // that here and use it throughout
    let country_code = country_code.to_lowercase();

    if let Some(holidays) = CachedHoliday::load(year, &country_code) {
        return Ok(holidays);
    }

    let client = reqwest::blocking::ClientBuilder::new()
        .timeout(Some(
            Duration::seconds(2)
                .try_into()
                .expect("std Duration can express 2 seconds"),
        ))
        .build()?;

    let body = client
        .get(uri_for(year, &country_code))
        .send()?
        .error_for_status()?
        .bytes()?;

    // returning an empty body with a 200 status code isn't the most convenient
    // possible way for the API to indicate that it doesn't know a particular
    // country code, but it's not the worst thing in the world.
    if body.is_empty() {
        return Err(Error::UnknownCountry);
    }

    let holidays = serde_json::from_slice(&body)?;

    let cache = CachedHoliday {
        fetched: OffsetDateTime::now_utc(),
        year,
        country_code,
        holidays,
    };
    cache.store()?;

    Ok(cache.holidays)
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
    #[error("no cache directory on this architecture")]
    NoCacheDir,
    #[error("io error manipulating cache")]
    Io(#[from] std::io::Error),
    #[error("json serialization")]
    Json(#[from] serde_json::Error),
}
