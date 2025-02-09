use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use iroh::{protocol::Router, Endpoint};
use iroh_blobs::{
    net_protocol::Blobs,
    rpc::client::blobs::{ReadAtLen, WrapOption},
    ticket::BlobTicket,
    util::SetTagOption,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create an endpoint, it allows creating and accepting
    // connections in the iroh p2p world
    let endpoint = Endpoint::builder().discovery_n0().bind().await?;

    let blobs = Blobs::memory().build(&endpoint);

    let router = Router::builder(endpoint)
        .accept(iroh_blobs::ALPN, blobs.clone())
        .spawn()
        .await?; // ALPN: Application Level Protocol Negotiation

    let blobs = blobs.client(); // Have a lot of things on the blobs protocol, the download
                                // could be paused and the hash now exactly where to start over
                                // to restart the download.

    let args = std::env::args().collect::<Vec<_>>();
    match &args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        [_cmd, "send", path] => {
            let abs_path = PathBuf::from(path).canonicalize()?;

            println!("Analyzing file.");

            let blob = blobs
                .add_from_path(abs_path, true, SetTagOption::Auto, WrapOption::NoWrap)
                .await?
                .finish()
                .await?;

            let node_id = router.endpoint().node_id();
            let ticket = BlobTicket::new(node_id.into(), blob.hash, blob.format)?;

            println!("File analyzed. Fetch this file by running:");
            println!("cargo run --example transfer -- receive {ticket} {path}");

            tokio::signal::ctrl_c().await?;
        }
        [_cmd, "receive", ticket, path] => {
            let path_buf = PathBuf::from(path);
            let ticket = BlobTicket::from_str(ticket)?;

            println!("Starting download.");

            blobs
                .download(ticket.hash(), ticket.node_addr().clone())
                .await?
                .finish()
                .await?;

            println!("Finished download.");

            println!("Copying to destination.");

            let mut file = tokio::fs::File::create(path_buf).await?;
            let mut reader = blobs.read_at(ticket.hash(), 0, ReadAtLen::All).await?;
            tokio::io::copy(&mut reader, &mut file).await?;

            println!("Finished copying.");
        }
        _ => {
            println!("Couldn't parse command line arguments.");
            println!("Usage:");
            println!("    # to send:");
            println!("    cargo run --example transfer -- send [FILE]");
            println!("    # this will print a ticket.");
            println!();
            println!("    # to receive:");
            println!("    cargo run --example transfer -- receive [TICKET] [FILE]");
        }
    }

    // Gracefully shut down the router
    println!("Shutting down.");
    router.shutdown().await?;

    Ok(())
}
