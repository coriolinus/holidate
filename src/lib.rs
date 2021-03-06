use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use time::{Date, Duration, OffsetDateTime};

/// How long a cached list of holidays is valid for, before hitting the API
/// again to check for updates.
const CACHE_FADEOUT: Duration = Duration::hours(24);

/// Type of holiday.
///
/// The Nager API specifies these types of holiday without explaining much about what their precise semantics are.
#[derive(Debug, parse_display::Display, Deserialize, Serialize)]
pub enum HolidayType {
    Public,
    Bank,
    School,
    Authorities,
    Optional,
    Observance,
}

/// A Holiday is a recurring officially named day in recognition or commemoration of a
/// certain event or celebration.
// The Nager API provides several other fields than these, but we don't care
// about them for this use case, and `serde_json` conveniently just ignores
// any fields which aren't present in the struct.
#[derive(Debug, Deserialize, Serialize)]
pub struct Holiday {
    /// The date of this instance of the holiday.
    pub date: Date,
    /// Holiday's name, in English.
    pub name: String,
    /// If this holiday is not celebrated throughout the nation in question, which sub-regions it is celebrated in.
    #[serde(deserialize_with = "deserialize_null_default")]
    pub counties: Vec<String>,
    /// What type(s) of holiday this is.
    ///
    /// May affect what things are closed on this holiday.
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

/// The `CacheManager` wraps a client, so that in the event we require several
/// requests, they can reuse the connection.
///
/// This typically happens when there aren't enough holidays left in the year
/// to fill the requested quantity, so we need to page forward and request the
/// subsequent year's.
struct CacheManager {
    client: reqwest::blocking::Client,
}

impl CacheManager {
    fn new() -> Result<Self, Error> {
        let two_seconds = Some(
            Duration::seconds(2)
                .try_into()
                .expect("std Duration can express 2 seconds"),
        );
        let client = reqwest::blocking::ClientBuilder::new()
            .https_only(true)
            .timeout(two_seconds)
            .tcp_keepalive(two_seconds)
            .build()?;

        Ok(Self { client })
    }

    fn get_holidays(&self, year: i32, country_code: &str) -> Result<Vec<Holiday>, Error> {
        // the cache only ever deals with lowercase country codes, so let's compute
        // that here and use it throughout
        let country_code = country_code.to_lowercase();

        if let Some(holidays) = CachedHoliday::load(year, &country_code) {
            return Ok(holidays);
        }

        let response = self.client.get(uri_for(year, &country_code)).send()?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            // It's pretty unclear when a fake/unknown country code will return
            // a 404 vs an empty body, but we have to handle both cases.
            return Err(Error::UnknownCountry);
        }
        let body = response.error_for_status()?.bytes()?;

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
}

/// Get the next several holidays in a specified country on or after a particular date.
pub fn next_holidays(
    country_code: &str,
    relative_to: Date,
    quantity: usize,
) -> Result<Vec<Holiday>, Error> {
    let mut year = relative_to.year();
    let mut holidays = Vec::new();
    let cache_manager = CacheManager::new()?;

    while holidays.len() < quantity {
        let new_holidays: Vec<Holiday> = cache_manager.get_holidays(year, country_code)?;
        holidays.extend(
            new_holidays
                .into_iter()
                .filter(|holiday| holiday.date >= relative_to),
        );
        year += 1;
    }
    holidays.truncate(quantity);
    Ok(holidays)
}

fn uri_for(year: i32, country_code: &str) -> String {
    format!("https://date.nager.at/api/v3/publicholidays/{year}/{country_code}")
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
