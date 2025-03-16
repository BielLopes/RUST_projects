use redis::{AsyncCommands, Client};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuração do cliente Valkey/Redis
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_tokio_connection().await?;

    // Chave a ser buscada
    let target_key = "user:123";

    // Buscar os dados usando HGETALL para hashes
    let result: HashMap<String, String> = con.hgetall(target_key).await?;

    if !result.is_empty() {
        println!("Dados recuperados para '{}':", target_key);
        for (field, value) in &result {
            println!("- {}: {}", field, value);
        }
    } else {
        println!("Chave '{}' não encontrada!", target_key);
    }

    Ok(())
}
