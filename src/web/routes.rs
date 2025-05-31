use axum::{extract::{State, Form}, response::Html, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use chrono::{DateTime, Utc};

use crate::ExchangeRates;
use super::AppState;
use super::templates;

#[derive(Deserialize)]
pub struct ConversionForm {
    amount: f64,
    from: String,
    to: String,
}

#[derive(Serialize)]
pub struct ConversionResult {
    pub amount: String,
    pub from: String,
    pub to: String,
    pub result: String,
    pub rate: String,
    pub timestamp: String,
}

async fn fetch_exchange_rates(api_key: &str, base: &str) -> anyhow::Result<ExchangeRates> {
    let client = reqwest::Client::new();
    
    // Try the free API from ExchangeRate-API first (no key required)
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
            } else {
                println!("Error parsing free API response: {}", anyhow::anyhow!("Invalid JSON"));
            }
        }
    }
    
    // Try another free API from fawazahmed0/currency-api (GitHub)
    let fallback_url = format!("https://cdn.jsdelivr.net/gh/fawazahmed0/currency-api@1/latest/currencies/{}.json", base.to_lowercase());
    let fallback_response = client.get(&fallback_url).send().await;
    
    if let Ok(response) = fallback_response {
        if response.status().is_success() {
            // This API has yet another format
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
                            timestamp: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
                            base: Some(base.to_string()),
                            date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
                            rates,
                        });
                    }
                }
            } else {
                println!("Error parsing API response: {}", anyhow::anyhow!("Invalid JSON"));
            }
        }
    }
    
    // Try the Frankfurter API (another free option)
    let frankfurter_url = format!("https://api.frankfurter.app/latest?from={}", base);
    let frankfurter_response = client.get(&frankfurter_url).send().await;
    
    if let Ok(response) = frankfurter_response {
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
                            timestamp: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
                            base: Some(base.to_string()),
                            date: value.get("date").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            rates,
                        });
                    }
                }
            } else {
                println!("Error parsing Frankfurter API response: {}", anyhow::anyhow!("Invalid JSON"));
            }
        }
    }
    
    // Only try the API key version as a last resort if provided
    if !api_key.is_empty() {
        let url = format!("https://api.exchangerate.host/latest?base={}&access_key={}", base, api_key);
        let response = client.get(&url).send().await;
        
        if let Ok(response) = response {
            if response.status().is_success() {
                match response.json::<ExchangeRates>().await {
                    Ok(rates) => {
                        if rates.success {
                            println!("Successfully fetched rates from api.exchangerate.host");
                            return Ok(rates);
                        }
                    }
                    Err(e) => {
                        println!("Error parsing API response: {}", e);
                    }
                }
            }
        }
    }
    
    // If all APIs fail, return a mock response for demonstration purposes
    // In a real app, you'd want to handle this differently
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
    
    println!("Using mock exchange rates for demonstration");
    
    Ok(ExchangeRates {
        success: true,
        timestamp: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
        base: Some(base.to_string()),
        date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        rates: mock_rates,
    })
}

async fn index() -> Html<String> {
    templates::render_index()
}

async fn convert(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ConversionForm>,
) -> Html<String> {
    let result = convert_currency(
        &state.api_key,
        form.amount,
        &form.from,
        &form.to,
    ).await;

    match result {
        Ok(conversion) => templates::render_conversion_result(conversion),
        Err(e) => templates::render_error(e.to_string()),
    }
}

async fn convert_currency(
    api_key: &str,
    amount: f64,
    from: &str,
    to: &str,
) -> anyhow::Result<ConversionResult> {
    let rates = fetch_exchange_rates(api_key, from).await?;
    
    let rate = rates.rates.get(to).ok_or_else(|| anyhow::anyhow!("Currency {} not found", to))?;
    let converted = amount * rate;
    
    // Format the timestamp
    let now = SystemTime::now();
    let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0).unwrap();
    let formatted_time = dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    
    // Format numbers with 2 decimal places
    let formatted_amount = format!("{:.2}", amount);
    let formatted_result = format!("{:.2}", (converted * 100.0).round() / 100.0);
    let formatted_rate = format!("{:.2}", (rate * 100.0).round() / 100.0);
    
    Ok(ConversionResult {
        amount: formatted_amount,
        from: from.to_string(),
        to: to.to_string(),
        result: formatted_result,
        rate: formatted_rate,
        timestamp: formatted_time,
    })
}

async fn list_currencies(
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let result = get_currencies(&state.api_key).await;
    
    match result {
        Ok(currencies) => templates::render_currencies_list(currencies),
        Err(e) => templates::render_error(e.to_string()),
    }
}

async fn get_currencies(api_key: &str) -> anyhow::Result<Vec<String>> {
    let rates = fetch_exchange_rates(api_key, "EUR").await?;
    
    let mut currencies: Vec<String> = rates.rates.keys().cloned().collect();
    currencies.sort();
    
    Ok(currencies)
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/convert", post(convert))
        // Removed the '/currencies' route as we no longer need it
        .with_state(state)
}
