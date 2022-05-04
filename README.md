# Holidate

Get the next several holidays for a given country from the API at <https://date.nager.at>.

## Design Notes

- Expanded the CLI a bit beyond the design requirements where extension was obvious.
- No uses of `unwrap` in this codebase. A few uses of `expect` where I felt falsification was unlikely.
- Line formatter for `Holiday` lives in `main.rs` instead of as a method on the type because it is only relevant to the CLI context.
- We use blocking requests because we can't reasonably speed up this API asynchronously, and blocking is simpler
- API is assumed to be good; there isn't particular handling for many years without a holiday, etc
- We use a `>=` relation when comparing the holiday's date against the current date; the spec was a bit ambiguous about whether we'd prefer a `>` relation there instead.
- We don't stick strictly to the policy "no more than one request to the API per 24 hours" because it is insufficient. Though we do cache the data, changes to the year or country can both require new requests.
