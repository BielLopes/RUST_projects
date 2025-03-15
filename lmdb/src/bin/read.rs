use lz4_flex::block::decompress_size_prepended;
use std::collections::HashMap;
use std::path::Path;

use bincode;
use lmdb::{Environment, Transaction};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuração do ambiente LMDB
    let env = Environment::new()
        .set_max_dbs(1)
        .set_map_size(1024 * 1024 * 1024)
        .open(Path::new("/home/gabriel/lmdb"))?;

    // Abrir a database
    let db = env.open_db(None)?;

    // Iniciar transação de leitura
    let txn = env.begin_ro_txn()?;

    // Chave a ser buscada
    let target_key = "user:123";

    // Buscar os dados brutos
    match txn.get(db, &target_key.as_bytes()) {
        Ok(raw_data) => {
            // Descomprimir os dados
            let decompressed = decompress_size_prepended(raw_data)?;
            // Desserializar o HashMap
            let config = bincode::config::standard()
                .with_fixed_int_encoding()
                .with_little_endian();
            let decoded: HashMap<String, String> =
                bincode::decode_from_slice(&decompressed, config)?.0;

            println!("Dados recuperados para '{}':", target_key);
            for (k, v) in &decoded {
                println!("- {}: {}", k, v);
            }
        }
        Err(lmdb::Error::NotFound) => {
            println!("Chave '{}' não encontrada!", target_key);
        }
        Err(e) => return Err(e.into()),
    }

    txn.commit()?;
    Ok(())
}
