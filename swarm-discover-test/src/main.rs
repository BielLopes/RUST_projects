use anyhow::Result;
use clap::Parser;
use if_addrs::get_if_addrs;
use swarm_discovery::Discoverer;
use tokio::runtime::Builder;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, help = "Nome do arquivo para o canal de descoberta")]
    file: String,
    #[arg(short, long, help = "Número do peer ID (1-99)")]
    id: u8,
    #[arg(short, long, help = "Porta de conexão")]
    port: u16,
}

impl Cli {
    fn validate(&self) {
        if self.id < 1 || self.id > 99 {
            eprintln!("ID deve estar entre 1 e 99.");
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    args.validate();

    let rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    let peer_id = format!("peer_{}", args.id);
    let addrs = get_if_addrs()
        .unwrap()
        .into_iter()
        .map(|i| i.addr.ip())
        .collect::<Vec<_>>();
    let channel_name = format!("file_{}", args.file);

    let _guard = Discoverer::new(channel_name, peer_id)
        .with_addrs(args.port, addrs)
        .with_callback(|peer_id, peer| {
            println!("Discovered {}: {:?}", peer_id, peer);
        })
        .spawn(rt.handle())
        .expect("discoverer spawn");

    tokio::signal::ctrl_c().await?;

    Ok(())
}
