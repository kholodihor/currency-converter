# Currency Converter

A Rust application that provides both CLI and web interfaces for currency conversion. It fetches real-time exchange rates from exchangerate.host API and allows you to convert between different currencies.

## Features

- List available currencies
- Convert amounts between different currencies
- Uses real-time exchange rates from exchangerate.host API

## Prerequisites

- Rust and Cargo installed
- An API key from [exchangerate.host](https://exchangerate.host)

## Setup

1. Clone this repository
2. Add your API key to the `.env` file:
   ```
   EXCHANGERATE_API_KEY=your_api_key_here
   ```
3. Build the project:
   ```
   cargo build --release
   ```

## Usage

### List available currencies

```
./target/release/currency-converter list
```

### Convert currency

```
./target/release/currency-converter convert <AMOUNT> <FROM> <TO>
```

Example:
```
./target/release/currency-converter convert 100 USD EUR
```

## License

MIT
