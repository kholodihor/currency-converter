# Currency Converter

A modern Rust application that provides both CLI and web interfaces for currency conversion. It fetches real-time exchange rates from multiple free APIs with fallback mechanisms and allows you to convert between different currencies.

## Technical Stack

- **Backend**: Rust with Axum web framework
- **Frontend**: HTMX and Tailwind CSS (dark theme with purple accents)
- **Async Runtime**: Tokio
- **Error Handling**: Anyhow
- **CLI Interface**: Clap
- **HTTP Client**: Reqwest
- **Serialization**: Serde

## Features

- **Multiple Currency Support**: Includes major currencies like USD, EUR, GBP, JPY, and additional currencies like PLN and UAH
- **Free API Integration**: Uses multiple free currency exchange APIs with fallback mechanisms:
  - Open Exchange Rates API (open.er-api.com)
  - Frankfurter API (api.frankfurter.app)
  - Fawaz Ahmed's Currency API (cdn.jsdelivr.net/gh/fawazahmed0/currency-api)
  - Mock data fallback if all APIs fail
- **Responsive UI**: Dark theme with purple accents
- **CLI and Web Interfaces**: Use as a command-line tool or web application

## Architecture

- **Modular Design**: Separation between CLI and web interfaces
- **Template Rendering**: Custom template system for HTML generation
- **Error Handling**: Comprehensive error handling with fallbacks
- **Asynchronous Processing**: Non-blocking API requests
- **Minimal JavaScript**: Uses HTMX for interactivity without heavy client-side JS

## Prerequisites

- Rust and Cargo installed

## Setup

1. Clone this repository
2. Build the project:
   ```
   cargo build --release
   ```



## Development

```bash
# Run in development mode
cargo run -- web

# Run tests
cargo test

# Build for production
cargo build --release
```

## License

MIT
