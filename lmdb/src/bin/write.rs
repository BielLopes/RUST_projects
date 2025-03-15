use std::collections::HashMap;
use std::path::Path;

use bincode;
use lmdb::{Environment, Transaction, WriteFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuração do ambiente LMDB
    let env = Environment::new()
        .set_max_dbs(1) // Máximo de databases
        .set_map_size(1024 * 1024 * 1024) // 1GB de tamanho máximo
        .open(Path::new("/home/gabriel/lmdb"))?; // Pasta para armazenamento

    // Abrir/Criar database
    let db = env.open_db(None)?;

    // Dados de exemplo
    let key = "user:123".to_string();

    // Valor inicial
    let mut value = HashMap::new();
    value.insert("nome".to_string(), "João Silva".to_string());
    value.insert("email".to_string(), "joao@exemplo.com".to_string());

    // Operação de INSERT
    let mut txn = env.begin_rw_txn()?;
    txn.put(
        db,
        &key.as_bytes(),
        &compress_data(&value),
        WriteFlags::empty(),
    )?;
    txn.commit()?;
    println!("Insert realizado com sucesso!");

    // Operação de UPDATE
    let mut txn = env.begin_rw_txn()?;

    // Novos valores
    value.insert("email".to_string(), "novo_email@exemplo.com".to_string());
    value.insert("telefone".to_string(), "11999999999".to_string());

    txn.put(
        db,
        &key.as_bytes(),
        &compress_data(&value),
        WriteFlags::empty(),
    )?;
    txn.commit()?;
    println!("Update realizado com sucesso!");

    Ok(())
}

fn compress_data(value: &HashMap<String, String>) -> Vec<u8> {
    let config = bincode::config::standard()
        .with_fixed_int_encoding()
        .with_little_endian();

    let serialized = bincode::encode_to_vec(value, config).unwrap();
    lz4_flex::block::compress_prepend_size(&serialized)
}
