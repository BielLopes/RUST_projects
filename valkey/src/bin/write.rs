use redis::{AsyncCommands, Client};

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuração do cliente Valkey/Redis
    let client = Client::open("redis://127.0.0.1/")?;

    // Conexão assíncrona multiplexada
    let mut con = client.get_multiplexed_tokio_connection().await?;

    // Dados de exemplo
    let key = "user:123";
    let mut fields = HashMap::new();
    fields.insert("nome", "João Silva");
    fields.insert("email", "joao@exemplo.com");

    // Operação de INSERT (HSET para hash)
    let vector: Vec<(&str, &str)> = fields.into_iter().collect();
    con.hset_multiple::<_, _, _, ()>(key, &vector).await?;
    println!("Insert realizado com sucesso!");

    // Operação de UPDATE
    let mut update_fields = HashMap::new();
    update_fields.insert("email", "novo_email@exemplo.com");
    update_fields.insert("telefone", "11999999999");

    // Experimento usando uma macro:
    macro_rules! fields {
        ($map:expr) => {
            &$map.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>()
        };
    }

    // let vector: Vec<(&str, &str)> = update_fields.into_iter().collect();
    con.hset_multiple::<_, _, _, ()>(key, fields!(update_fields))
        .await?;
    println!("Update realizado com sucesso!");

    Ok(())
}
