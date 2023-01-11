mod configuration;
mod download;

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::new()
        .default_filter_or("modrinth_downloader=info")
    )
    .init();

    let c = configuration::get_config();

    let d = download::Downloader::new(c.mod_path, c.version, c.loader);

    for m in c.mod_ids {
        d.download(m).unwrap();
    }
}
