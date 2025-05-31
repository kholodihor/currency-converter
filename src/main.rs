use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
// Removed unused import: use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;

mod web;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available currencies
    List,
    /// Convert from one currency to another
    Convert {
        /// Amount to convert
        amount: f64,
        /// Source currency (e.g., USD)
        from: String,
        /// Target currency (e.g., EUR)
        to: String,
    },
    /// Start web server with UI
    Web {
        /// Port to run the web server on
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRates {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    pub rates: HashMap<String, f64>,
}

async fn fetch_exchange_rates(_api_key: &str, base: &str) -> Result<ExchangeRates> {
    let client = Client::new();
    
    // Try free API from ExchangeRate-API (no key required)
    let free_url = format!("https://open.er-api.com/v6/latest/{}", base);
    let free_response = client.get(&free_url).send().await;
    
    if let Ok(response) = free_response {
        if response.status().is_success() {
            // The free API has a different response format, so we need to parse it differently
            let response_text = response.text().await.unwrap_or_default();
            
            // Try to parse the response
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(rates_obj) = value.get("rates") {
                    if let Some(rates_map) = rates_obj.as_object() {
                        let mut rates = HashMap::new();
                        
                        for (currency, rate) in rates_map {
                            if let Some(rate_val) = rate.as_f64() {
                                rates.insert(currency.clone(), rate_val);
                            }
                        }
                        
                        println!("Successfully fetched rates from open.er-api.com");
                        return Ok(ExchangeRates {
                            success: true,
                            timestamp: value.get("time_last_update_unix").and_then(|v| v.as_u64()),
                            base: Some(base.to_string()),
                            date: value.get("time_last_update_utc").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            rates,
                        });
                    }
                }
            }
        }
    }
    
    // Try another free API as fallback (Frankfurter)
    let fallback_url = format!("https://api.frankfurter.app/latest?from={}", base);
    let fallback_response = client.get(&fallback_url).send().await;
    
    if let Ok(response) = fallback_response {
        if response.status().is_success() {
            let response_text = response.text().await.unwrap_or_default();
            
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(rates_obj) = value.get("rates") {
                    if let Some(rates_map) = rates_obj.as_object() {
                        let mut rates = HashMap::new();
                        
                        for (currency, rate) in rates_map {
                            if let Some(rate_val) = rate.as_f64() {
                                rates.insert(currency.clone(), rate_val);
                            }
                        }
                        
                        println!("Successfully fetched rates from api.frankfurter.app");
                        return Ok(ExchangeRates {
                            success: true,
                            timestamp: Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
                            base: Some(base.to_string()),
                            date: value.get("date").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            rates,
                        });
                    }
                }
            }
        }
    }
    
    // Try yet another free API as fallback (fawazahmed0/currency-api)
    let today = chrono::Utc::now().format("%Y-%m-%d");
    let currency_api_url = format!("https://cdn.jsdelivr.net/gh/fawazahmed0/currency-api@1/latest/currencies/{}.json", base.to_lowercase());
    let currency_api_response = client.get(&currency_api_url).send().await;
    
    if let Ok(response) = currency_api_response {
        if response.status().is_success() {
            let response_text = response.text().await.unwrap_or_default();
            
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(rates_obj) = value.get(base.to_lowercase()) {
                    if let Some(rates_map) = rates_obj.as_object() {
                        let mut rates = HashMap::new();
                        
                        for (currency, rate) in rates_map {
                            if let Some(rate_val) = rate.as_f64() {
                                rates.insert(currency.to_uppercase(), rate_val);
                            }
                        }
                        
                        println!("Successfully fetched rates from fawazahmed0/currency-api");
                        return Ok(ExchangeRates {
                            success: true,
                            timestamp: Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
                            base: Some(base.to_string()),
                            date: Some(today.to_string()),
                            rates,
                        });
                    }
                }
            }
        }
    }
    
    // If all APIs fail, return a mock response for demonstration purposes
    println!("All API requests failed. Using mock data.");
    
    let mut mock_rates = HashMap::new();
    mock_rates.insert("USD".to_string(), 1.08);
    mock_rates.insert("EUR".to_string(), 1.0);
    mock_rates.insert("GBP".to_string(), 0.85);
    mock_rates.insert("JPY".to_string(), 160.0);
    mock_rates.insert("CAD".to_string(), 1.47);
    mock_rates.insert("AUD".to_string(), 1.63);
    mock_rates.insert("CHF".to_string(), 0.97);
    mock_rates.insert("CNY".to_string(), 7.8);
    mock_rates.insert("PLN".to_string(), 4.26);
    mock_rates.insert("UAH".to_string(), 42.5);
    
    // Convert rates if base is not EUR
    if base != "EUR" {
        let base_rate = *mock_rates.get(base).unwrap_or(&1.0);
        let mut converted_rates = HashMap::new();
        for (currency, rate) in mock_rates.iter() {
            converted_rates.insert(currency.clone(), rate / base_rate);
        }
        mock_rates = converted_rates;
    }
    
    Ok(ExchangeRates {
        success: true,
        timestamp: Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
        base: Some(base.to_string()),
        date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        rates: mock_rates,
    })
}

async fn list_currencies(api_key: &str) -> Result<()> {
    let rates = fetch_exchange_rates(api_key, "EUR").await?;
    
    println!("Available currencies:
");
    let mut currencies: Vec<_> = rates.rates.keys().collect();
    currencies.sort();
    
    for currency in currencies {
        println!("{}", currency);
    }
    
    Ok(())
}

async fn convert_currency(api_key: &str, amount: f64, from: &str, to: &str) -> Result<()> {
    let rates = fetch_exchange_rates(api_key, from).await?;
    
    let rate = rates.rates.get(to).context(format!("Currency {} not found", to))?;
    let converted = amount * rate;
    
    println!("{} {} = {:.2} {}", amount, from, converted, to);
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for better logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();
    
    // We don't need an API key since we're using free APIs
    let api_key = "".to_string();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::List => {
            println!("Fetching available currencies...");
            let result = list_currencies(&api_key).await;
            
            if let Err(e) = result {
                println!("Error: {}", e);
                println!("
Possible causes:
- Check your internet connection
- The API service might be temporarily unavailable");
            }
        }
        Commands::Convert { amount, from, to } => {
            convert_currency(&api_key, *amount, from, to).await?;
        }
        Commands::Web { port } => {
            // Start web server
            let addr = SocketAddr::from(([0, 0, 0, 0], *port));
            println!("Starting web server on port {}...", port);
            println!("Open your browser and navigate to http://localhost:{}", port);
            println!("Press Ctrl+C to stop the server");

            // Create the application using the existing function
            let app = web::create_app(api_key.clone()).await;

            // Start the server
            match TcpListener::bind(&addr).await {
                Ok(listener) => {
                    println!("Server started successfully");
                    axum::serve(listener, app).await?;
                }
                Err(e) => {
                    println!("Error: Failed to bind to address

Caused by:
    {}", e);
                }
            }
        }
    }

    Ok(())
}
