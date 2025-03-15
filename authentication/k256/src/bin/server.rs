use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use hex;
use hmac::{Hmac, Mac};
use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use rand::{rngs::OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Chave secreta usada para HMAC – somente o servidor conhece.
const HMAC_SECRET: &[u8] = b"supersecretkey";

#[derive(Serialize, Deserialize, Debug)]
struct ChallengeToken {
    /// Desafio aleatório (hex string)
    challenge: String,
    /// Nonce gerado (hex string)
    nonce: String,
    /// Timestamp (segundos desde Unix Epoch)
    timestamp: u64,
    /// HMAC calculado sobre (challenge || transaction_hash || nonce || timestamp)
    hmac: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Transaction {
    nome: String,
    hash: String,
}

/// Requisição para GET /desafio – contém o hash da transação.
#[derive(Deserialize)]
struct DesafioRequest {
    transaction_hash: String,
}

/// Endpoint que gera o desafio.
#[get("/desafio")]
async fn get_desafio(body: web::Bytes) -> impl Responder {
    // Extrai o hash da transação do corpo JSON.
    let req: DesafioRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return HttpResponse::BadRequest().body(format!("Requisição inválida: {}", e)),
    };

    // Gera 32 bytes aleatórios para o desafio.
    let mut challenge_bytes = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut challenge_bytes)
        .expect("Erro ao gerar desafio aleatório");
    let challenge_hex = hex::encode(challenge_bytes);

    // Gera um nonce de 16 bytes.
    let mut nonce_bytes = [0u8; 16];
    OsRng
        .try_fill_bytes(&mut nonce_bytes)
        .expect("Erro ao gerar nonce aleatório");
    let nonce_hex = hex::encode(nonce_bytes);

    // Obtém o timestamp atual.
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Decodifica o hash da transação enviado pelo cliente (em hex).
    let transaction_hash_bytes = match hex::decode(&req.transaction_hash) {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::BadRequest().body(format!("Hash da transação inválido: {}", e))
        }
    };

    // Calcula o HMAC sobre: challenge_bytes || transaction_hash_bytes || nonce_bytes || timestamp
    let mut mac =
        HmacSha256::new_from_slice(HMAC_SECRET).expect("HMAC pode usar chave de qualquer tamanho");
    mac.update(&challenge_bytes);
    mac.update(&transaction_hash_bytes);
    mac.update(&nonce_bytes);
    mac.update(&timestamp.to_be_bytes());
    let result = mac.finalize().into_bytes();
    let hmac_hex = hex::encode(result);

    let token = ChallengeToken {
        challenge: challenge_hex,
        nonce: nonce_hex,
        timestamp,
        hmac: hmac_hex,
    };

    HttpResponse::Ok().json(token)
}

/// Requisição para POST /transacao.
#[derive(Deserialize)]
struct TransacaoRequest {
    token: ChallengeToken,
    /// Assinatura do cliente sobre o campo `challenge` (hex string)
    signature: String,
    /// Chave pública do cliente (hex string)
    public_key: String,
    transaction: Transaction,
}

/// Endpoint que valida a transação.
#[post("/transacao")]
async fn post_transacao(req: web::Json<TransacaoRequest>) -> impl Responder {
    let token = &req.token;

    // Verifica se o token não expirou (exemplo: válido por 5 minutos)
    let current_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if current_ts > token.timestamp + 300 {
        return HttpResponse::BadRequest().body("Token expirado");
    }

    // Calcula o hash da transação a partir dos dados (aqui: SHA256(nome + hash))
    let mut hasher = Sha256::new();
    hasher.update(req.transaction.nome.as_bytes());
    hasher.update(req.transaction.hash.as_bytes());
    let computed_tx_hash = hasher.finalize();
    let computed_tx_hash_hex = hex::encode(&computed_tx_hash);

    // Recalcula o HMAC usando o challenge, o hash da transação (calculado), o nonce e o timestamp.
    let challenge_bytes = match hex::decode(&token.challenge) {
        Ok(b) => b,
        Err(_) => return HttpResponse::BadRequest().body("Formato inválido no challenge do token"),
    };
    let nonce_bytes = match hex::decode(&token.nonce) {
        Ok(b) => b,
        Err(_) => return HttpResponse::BadRequest().body("Formato inválido no nonce do token"),
    };
    let computed_tx_hash_bytes = match hex::decode(&computed_tx_hash_hex) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::BadRequest().body("Erro ao decodificar o hash da transação")
        }
    };

    let mut mac =
        HmacSha256::new_from_slice(HMAC_SECRET).expect("HMAC pode usar chave de qualquer tamanho");
    mac.update(&challenge_bytes);
    mac.update(&computed_tx_hash_bytes);
    mac.update(&nonce_bytes);
    mac.update(&token.timestamp.to_be_bytes());
    let expected_hmac = mac.finalize().into_bytes();
    let expected_hmac_hex = hex::encode(expected_hmac);

    if expected_hmac_hex != token.hmac {
        return HttpResponse::BadRequest().body("Token inválido (HMAC não confere)");
    }

    // Verifica a assinatura do cliente sobre o challenge.
    let public_key_bytes = match hex::decode(&req.public_key) {
        Ok(b) => b,
        Err(_) => return HttpResponse::BadRequest().body("Formato inválido na chave pública"),
    };
    let verifying_key = match VerifyingKey::from_sec1_bytes(&public_key_bytes) {
        Ok(key) => key,
        Err(_) => return HttpResponse::BadRequest().body("Chave pública inválida"),
    };

    let signature_bytes = match hex::decode(&req.signature) {
        Ok(b) => b,
        Err(_) => return HttpResponse::BadRequest().body("Formato inválido na assinatura"),
    };
    let signature = match Signature::from_slice(&signature_bytes) {
        Ok(sig) => sig,
        Err(_) => return HttpResponse::BadRequest().body("Assinatura inválida"),
    };

    // A assinatura deve ser feita sobre o campo `challenge` (convertido para bytes).
    if verifying_key.verify(&challenge_bytes, &signature).is_err() {
        return HttpResponse::BadRequest().body("Falha na verificação da assinatura");
    }

    // Se tudo estiver correto, chama (simbolicamente) a função de armazenamento no LMDB.
    save_to_lmdb(&req.public_key, &req.transaction);

    HttpResponse::Ok().body("Transação aceita")
}

/// Stub para salvar no LMDB.
fn save_to_lmdb(public_key: &str, transaction: &Transaction) {
    println!(
        "Salvando no LMDB: public_key = {}, transaction = {:?}",
        public_key, transaction
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Servidor rodando em http://localhost:8080");
    HttpServer::new(|| App::new().service(get_desafio).service(post_transacao))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
