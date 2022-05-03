use time::Date;

#[derive(Debug, parse_display::FromStr, parse_display::Display)]
pub enum HolidayType {
    Public,
    Bank,
    School,
    Authorities,
    Optional,
    Observance,
}

pub struct Holiday {
    pub date: Date,
    pub name: String,
    pub counties: Vec<String>,
    pub types: Vec<HolidayType>,
}

pub fn next_holidays(
    country: &str,
    relative_to: Date,
    quantity: u32,
) -> Result<Vec<Holiday>, Error> {
    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown country code")]
    UnknownCountry,
}
