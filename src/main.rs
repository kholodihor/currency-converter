use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
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

async fn fetch_exchange_rates(api_key: &str, base: &str) -> Result<ExchangeRates> {
    // Try the primary API endpoint
    let primary_url = format!(
        "https://api.exchangerate.host/latest?access_key={}&base={}",
        api_key, base
    );
    
    let client = Client::new();
    let response = match client.get(&primary_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                resp
            } else {
                println!("Primary API failed with status: {}, trying fallback...", resp.status());
                // Try fallback API if primary fails
                let fallback_url = format!(
                    "https://api.exchangeratesapi.io/latest?access_key={}&base={}",
                    api_key, base
                );
                client.get(&fallback_url).send().await?
            }
        },
        Err(e) => {
            println!("Primary API request error: {}, trying fallback...", e);
            // Try fallback API if primary fails
            let fallback_url = format!(
                "https://api.exchangeratesapi.io/latest?access_key={}&base={}",
                api_key, base
            );
            client.get(&fallback_url).send().await?
        }
    };
    
    if !response.status().is_success() {
        anyhow::bail!("All API requests failed. Last status: {}", response.status());
    }
    
    let rates: ExchangeRates = response.json().await?;
    if !rates.success {
        anyhow::bail!("API returned unsuccessful response");
    }
    
    Ok(rates)
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
    if let Err(e) = dotenv::dotenv() {
        println!("Note: .env file not found or couldn't be loaded: {}", e);
        println!("You can create a .env file with EXCHANGERATE_API_KEY=your_key_here
");
    }
    
    // Get API key from environment variable
    let api_key = match env::var("EXCHANGERATE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Note: EXCHANGERATE_API_KEY environment variable not set.");
            println!("The application will use free APIs and fallback to mock data if needed.
");
            println!("For more reliable results, you can set the API key using one of these methods:
");
            println!("1. Create a .env file in the project directory with:
   EXCHANGERATE_API_KEY=your_api_key_here
");
            println!("2. Set the environment variable before running:
   export EXCHANGERATE_API_KEY=your_api_key_here
");
            println!("You can get a free API key from https://exchangerate.host
");
            "".to_string() // Return empty string instead of exiting
        }
    };
    
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
- Verify your API key is correct
- The currency codes might be invalid
- The API service might be temporarily unavailable");
            }
        }
        Commands::Convert { amount, from, to } => {
            println!("Converting {} {} to {}...", amount, from.to_uppercase(), to.to_uppercase());
            let result = convert_currency(&api_key, *amount, &from.to_uppercase(), &to.to_uppercase()).await;
            
            if let Err(e) = result {
                println!("Error: {}", e);
                println!("
Possible causes:
- Check your internet connection
- Verify your API key is correct
- The currency codes might be invalid
- The API service might be temporarily unavailable");
            }
        }
        Commands::Web { port } => {
            // Start the web server
            println!("Starting web server on port {}...", port);
            println!("Open your browser and navigate to http://localhost:{}", port);
            println!("Press Ctrl+C to stop the server");
            
            let app = web::create_app(api_key.clone()).await;
            let addr = SocketAddr::from(([127, 0, 0, 1], *port));
            
            // Create a TCP listener
            let listener = TcpListener::bind(addr).await
                .context("Failed to bind to address")?;
            
            println!("Server started successfully");
            
            // Start the server
            axum::serve(listener, app)
                .await
                .context("Failed to start server")?;
        }
    }
    
    Ok(())
}
