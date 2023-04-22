use futures::{stream::FuturesUnordered, StreamExt};

use color_eyre::Result;

mod configuration;
mod download;
mod hash;
mod modrinth;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().expect("Couldn't setup error logging!");

    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "modrinth_downloader=info,tracing_unwrap=error".into()),
        )
        .init();

    let c = configuration::get_config().await?;

    let d = download::Downloader::new(c.mod_path, c.version, c.loader);

    let mut futures = FuturesUnordered::new();

    for i in c.mod_ids {
        futures.push(d.download(i));
    }

    while let Some(_) = futures.next().await {}

    Ok(())
}
