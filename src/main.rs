use reqwest::Client;
use serde::Deserialize;
use serde_yaml;
use serde_json;
use std::fs;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Config {
    wallets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SolanaBalanceResponse {
    result: Option<SolanaBalanceResult>,
}

#[derive(Debug, Deserialize)]
struct SolanaBalanceResult {
    value: u64,
}

const LAMPORT_PER_SOL: u64 = 1_000_000_000;

async fn get_balance(client: &Client, address: &str) -> Result<u64, Box<dyn Error>> {
    let url = format!("https://api.mainnet-beta.solana.com");
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [address]
    });

    let response = client.post(&url)
        .json(&body)
        .send()
        .await?;

    let solana_response: SolanaBalanceResponse = response.json().await?;

    match solana_response.result {
        Some(result) => Ok(result.value),
        None => Err("Failed to get balance".into()),
    }
}

async fn get_all_balances(client: &Client, wallets: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut tasks = Vec::new();

    for wallet in wallets {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            match get_balance(&client, &wallet).await {
                Ok(balance) => println!("Balance for wallet {}: {} lamports", wallet, balance / LAMPORT_PER_SOL),
                Err(err) => eprintln!("Error getting balance for {}: {}", wallet, err),
            }
        }));
    }

    // Wait for all tasks to finish
    for task in tasks {
        task.await.unwrap();
    }

    Ok(())
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let config_file = fs::read_to_string("config.yaml")?;
    let config: Config = serde_yaml::from_str(&config_file)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;

    let client = Client::new();

    get_all_balances(&client, config.wallets).await?;

    Ok(())
}