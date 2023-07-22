# modrinth_downloader
Download a list of mods into a mods folder while checking SHASUMS

# Usage
Designed for use in docker container.

- Download the latest github release.
- Mount a configuration (`examples/config.toml`) in /config/config.toml. (path can be changed using CONFIG_PATH environment variable)
- Run the program `modrinth-downloader`. (no cli options exist)