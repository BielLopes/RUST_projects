use hex;
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use rand_core::OsRng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
struct ChallengeToken {
    challenge: String,
    nonce: String,
    timestamp: u64,
    hmac: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    nome: String,
    hash: String,
}

/// Requisição para GET /desafio.
#[derive(Serialize, Deserialize, Debug)]
struct DesafioRequest {
    transaction_hash: String,
}

/// Requisição para POST /transacao.
#[derive(Serialize, Deserialize, Debug)]
struct TransacaoRequest {
    token: ChallengeToken,
    /// Assinatura do cliente sobre o challenge (hex string)
    signature: String,
    /// Chave pública do cliente (hex string)
    public_key: String,
    transaction: Transaction,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    // Gera uma chave privada aleatória e obtém a chave pública correspondente.
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    // Cria uma transação constante.
    let transaction = Transaction {
        nome: "Alice".to_string(),
        hash: "12345".to_string(),
    };

    // Calcula o hash SHA256 da transação (aqui: SHA256(nome + hash)).
    let mut hasher = Sha256::new();
    hasher.update(transaction.nome.as_bytes());
    hasher.update(transaction.hash.as_bytes());
    let tx_hash = hasher.finalize();
    let tx_hash_hex = hex::encode(tx_hash);

    // Envia uma requisição GET para /desafio com o hash no corpo (em JSON).
    let desafio_url = "http://localhost:8080/desafio";
    let desafio_req = DesafioRequest {
        transaction_hash: tx_hash_hex.clone(),
    };

    let resp = client
        .get(desafio_url)
        .body(serde_json::to_string(&desafio_req)?)
        .send()
        .await?;

    if !resp.status().is_success() {
        eprintln!("Erro ao obter desafio: {}", resp.status());
        return Ok(());
    }

    let token: ChallengeToken = resp.json().await?;
    println!("Token recebido: {:?}", token);

    // O cliente assina o challenge recebido.
    let challenge_bytes = hex::decode(&token.challenge)?;
    let signature: Signature = signing_key.sign(&challenge_bytes);
    let signature_hex = hex::encode(&signature.to_bytes());

    // Prepara a requisição POST para /transacao.
    let transacao_req = TransacaoRequest {
        token,
        signature: signature_hex,
        public_key: hex::encode(verifying_key.to_sec1_bytes()),
        transaction,
    };

    let transacao_url = "http://localhost:8080/transacao";
    let resp = client
        .post(transacao_url)
        .json(&transacao_req)
        .send()
        .await?;

    let resp_text = resp.text().await?;
    println!("Resposta do servidor: {}", resp_text);

    Ok(())
}
