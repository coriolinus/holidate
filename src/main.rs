use std::str::FromStr;

use holidate::Holiday;
use itertools::Itertools;
use structopt::StructOpt;
use time::{macros::format_description, Date};

#[derive(Debug)]
struct ParseableDate(Date);

impl FromStr for ParseableDate {
    type Err = time::error::Parse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        time::Date::parse(s, format_description!("[year]-[month]-[day]")).map(ParseableDate)
    }
}

#[derive(Debug, StructOpt)]
struct Options {
    /// Date relative to which we find the next holidays.
    ///
    /// Defaults to today. Otherwise must be in "[year]-[month]-[day]" format.
    #[structopt(short, long)]
    relative_to: Option<ParseableDate>,

    /// How many holidays to retrieve.
    #[structopt(short, long, default_value = "5")]
    number: u32,

    /// Country code for which to look up holidays.
    ///
    /// Must be a member of the list at <https://date.nager.at/Country>.
    country_code: String,
}

impl Options {
    fn relative_to(&self) -> Date {
        match self.relative_to {
            Some(ParseableDate(date)) => date,
            None => time::OffsetDateTime::now_local()
                .expect("local tz offset should be discoverable on this machine")
                .date(),
        }
    }
}

/// Join a slice of stringable things into a comma-separated string containing the list.
fn comma_sep(items: &[impl ToString]) -> String {
    items.iter().map(ToString::to_string).join(", ")
}

/// Format a `Holiday` on a line.
///
/// This lives here and not as a method on `Holiday` because it's specific to
/// this particular CLI application; it's not particularly generalizable.
fn print_holiday(
    Holiday {
        date,
        name,
        counties,
        types,
    }: &Holiday,
) {
    let counties = comma_sep(&counties);
    let types = comma_sep(&types);

    println!("{date} {name:40} {counties:25} {types}")
}

fn main() -> color_eyre::eyre::Result<()> {
    // setup panic and error handlers
    color_eyre::install()?;

    let options = Options::from_args();
    for holiday in
        holidate::next_holidays(&options.country_code, options.relative_to(), options.number)?
    {
        print_holiday(&holiday);
    }

    Ok(())
}
