# Task

Write a cli that fetches public holidays from the existing API, https://date.nager.at, and show the next 5 occurring holidays.
- In order to avoid fetching the data too frequently, the endpoint shouldn't be called more than once a day.
- The country code should be passed as a cli argument.
- The output should contain the following information: Date, name, counties, types

## API

[docs](https://date.nager.at/Api)

### Request

```http
GET /api/v3/PublicHolidays/{Year}/{CountryCode}
```

### Response

```json
[
   {
      "date": "2017-01-01",
      "localName": "Neujahr",
      "name": "New Year's Day",
      "countryCode": "AT",
      "fixed": true,
      "global": true,
      "counties": null,
      "launchYear": 1967,
      "types": [
         "Public"
      ]
   },
   ...
]
```
