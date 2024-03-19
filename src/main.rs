use std::sync::Arc;

use color_eyre::Result;

mod configuration;
mod download;
mod hash;
mod modrinth;

fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "modrinth_downloader=info".into()),
        )
        .init();

    let c = configuration::get_config();

    let d = Arc::new(download::Downloader::new(c.mod_path, c.version, c.loader));

    std::thread::scope(|s| {
        for i in c.mod_ids {
            let dler = Arc::clone(&d);
            s.spawn(move || {
                dler.download(i);
            });
        }
    });

    Ok(())
}
