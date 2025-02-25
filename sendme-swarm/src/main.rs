use anyhow::Result;
use clap::{Parser, Subcommand};
use data_encoding::BASE32_NOPAD_NOCASE as BASE32;
use if_addrs::get_if_addrs;
use iroh::{protocol::Router, Endpoint, NodeAddr, NodeId};
use iroh_blobs::{
    net_protocol::Blobs,
    rpc::client::blobs::{ReadAtLen, WrapOption},
    util::SetTagOption,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use swarm_discovery::Discoverer;
use tokio::runtime::Builder;

type PeerMap = Arc<Mutex<HashMap<String, Vec<NodeAddr>>>>;
// const BASE36: &str = "0123456789abcdefghijkmnopqrstuvwxyz";

#[derive(Parser, Debug)]
#[command(version, about = "Swarm Discovery CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value_t = 1, help = "Número do peer ID (1-99)")]
    id: u8,
    #[arg(short, long, default_value_t = 1234, help = "Porta de conexão")]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Send {
        #[arg(help = "Path to the file to send")]
        path: PathBuf,
    },
    Receive {
        #[arg(help = "Hash of the file to receive")]
        blob_hash: String,
        #[arg(help = "Path to save the received file")]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let endpoint = Endpoint::builder().discovery_n0().bind().await?;
    let blobs = Blobs::memory().build(&endpoint);
    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs.clone())
        .spawn()
        .await?;
    let blobs = blobs.client();

    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    let addrs = get_if_addrs()
        .unwrap()
        .into_iter()
        .map(|i| i.addr.ip())
        .collect::<Vec<_>>();

    let peer_map: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    match args.command {
        Commands::Send { path } => {
            let abs_path = path.canonicalize()?;
            println!("Analyzing file: {}.", abs_path.display());
            let blob = blobs
                .add_from_path(abs_path, true, SetTagOption::Auto, WrapOption::NoWrap)
                .await?
                .finish()
                .await?;
            let blob_hash = blob.hash.to_string();

            let channel_name = format!("file_{}", blob_hash);

            let node_id = router.endpoint().node_id();
            let local_peer_id = BASE32.encode(node_id.as_bytes());

            let _guard = Discoverer::new(channel_name, local_peer_id.clone())
                .with_addrs(args.port, addrs)
                .with_callback(move |peer_id, peer| {
                    if peer_id != local_peer_id {
                        println!("Discovered peer {peer_id} at {:?}", peer);
                    }
                })
                .spawn(rt.handle())
                .expect("Discoverer spawn");

            println!("File analyzed. Share this hash: {blob_hash}");
            tokio::signal::ctrl_c().await?;
        }
        Commands::Receive { blob_hash, path } => {
            let channel_name = format!("file_yzvqh5gtay43prcpvoe2if63lk3pk3rnsf22ihq2t434vtajo6gq");
            let local_peer_id = format!(
                "gitfreedom_peer_{}",
                router
                    .endpoint()
                    .node_id()
                    .to_string()
                    .chars()
                    .take(10)
                    .collect::<String>()
            );
            let peer_map_clone = Arc::clone(&peer_map);
            let blob_hash_clone = blob_hash.clone();

            let _guard = Discoverer::new(channel_name, local_peer_id.clone())
                .with_addrs(args.port, addrs)
                .with_callback(move |peer_id, peer| {
                    if peer_id != local_peer_id {
                        println!("Discovered peer {peer_id} at {:?}", peer);
                        let decoded_node_id = BASE32
                            .decode(peer_id.as_bytes())
                            .expect("Failed to recovery with base 32!");
                        let hex_node: String = decoded_node_id
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect();
                        println!("Recovery node_id: {hex_node}");
                        let mut fixed_array = [0u8; 32];
                        let len = decoded_node_id.len().min(32);
                        fixed_array[..len].copy_from_slice(&decoded_node_id[..len]);

                        if let Ok(node_id) = NodeId::from_bytes(&fixed_array) {
                            let mut map = peer_map_clone.lock().unwrap();
                            println!("Adding: {node_id}");
                            map.entry(blob_hash_clone.clone())
                                .or_insert_with(Vec::new)
                                .push(NodeAddr::from(node_id));
                        };
                    }
                })
                .spawn(rt.handle())
                .expect("Discoverer spawn");

            println!("Looking for peers with file {blob_hash}.");
            tokio::time::sleep(std::time::Duration::from_secs(20)).await;

            let peers = {
                let map = peer_map.lock().unwrap();
                map.get(&blob_hash).cloned()
            };
            println!("{peers:?}");

            if let Some(peers) = peers {
                for peer in peers {
                    println!("Trying to download from {}", peer.node_id);
                    if let Ok(_) = blobs
                        .download(blob_hash.parse()?, peer.clone())
                        .await?
                        .finish()
                        .await
                    {
                        println!("Download complete from {}.", peer.node_id);
                        break;
                    }
                }
            } else {
                println!("No peers found for file {blob_hash}.");
            }

            let mut file = tokio::fs::File::create(path).await?;
            let mut reader = blobs.read_at(blob_hash.parse()?, 0, ReadAtLen::All).await?;
            tokio::io::copy(&mut reader, &mut file).await?;

            println!("Download complete.");
        }
    };

    println!("Shutting down.");
    router.shutdown().await?;
    Ok(())
}
