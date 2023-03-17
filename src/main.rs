use futures::{stream::FuturesUnordered, StreamExt};

mod configuration;
use configuration::ConfigurationError;
mod download;
mod hash;
mod modrinth;

#[tokio::main]
async fn main() -> Result<(), std::io::ErrorKind> {
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
            match error {
                ConfigurationError::IOError { error, path } => {
                    tracing::error!(path, %error, "Couldn't get configuration!");
                    std::process::exit(error.kind() as i32)
                },
                ConfigurationError::TOMLError { error, path } => {
                    tracing::error!(path, %error, "Couldn't parse configuration!");
                    std::process::exit(ErrorKind::InvalidData as i32);
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
