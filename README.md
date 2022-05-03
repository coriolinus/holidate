# Holidate

Get the next several holidays for a given country from the API at <https://date.nager.at>.

## Design Notes

- Expanded the CLI a bit beyond the design requirements because it was an obvious extension.
- No uses of `unwrap` in this codebase. A few uses of `expect` where I felt falsification was unlikely.
- Line formatter for `Holiday` lives in `main.rs` instead of as a method on the type because it is only relevant to the CLI context.
