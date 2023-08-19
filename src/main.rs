use std::thread::spawn;
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
                .unwrap_or_else(|_| "modrinth_downloader=info,tracing_unwrap=error".into()),
        )
        .init();

    let c = configuration::get_config()?;

    let d = Arc::new(download::Downloader::new(c.mod_path, c.version, c.loader));

    let mut handles = Vec::new();

    for i in c.mod_ids {
        let dler = Arc::clone(&d);
        handles.push(spawn(move || {
            dler.download(i);
        }))
    }

    for h in handles {
        h.join().unwrap();
    }

    Ok(())
}
