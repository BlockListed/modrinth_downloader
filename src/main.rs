use futures::{stream::FuturesUnordered, StreamExt};

mod configuration;
mod download;
mod hash;
mod modrinth;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "modrinth_downloader=info,tracing_unwrap=error".into()),
        )
        .init();

    let c = match configuration::get_config().await {
        Ok(x) => x,
        Err(error) => {
            use std::io::ErrorKind;
            return match error.kind() {
                ErrorKind::InvalidData => {
                    tracing::error!("Invalid TOML data!");
                    Err(error)
                },
                ErrorKind::NotFound => {
                    tracing::error!("Could not find config file! You can set a custom config file path by setting the `CONFIG_PATH` environment variable.");
                    Err(error)
                },
                _ => {
                    tracing::error!("Could not open config file! You can set a custom config file path by setting the `CONFIG_PATH` environment variable.");
                    Err(error)
                }
            }
        }
    };

    let d = download::Downloader::new(c.mod_path, c.version, c.loader);

    let futures = FuturesUnordered::new();

    for i in c.mod_ids {
        futures.push(d.download(i));
    }

    let _: Vec<()> = futures.collect().await;

    Ok(())
}
