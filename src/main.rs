mod config;
use config::load_config;
mod download;
use download::download_channel;
mod link;
use link::link_channel_playlists;

fn main() -> anyhow::Result<()> {
    //TODO: take the path as argument and check exists, else use dirs::config_dir()
    let cfg = load_config("config.toml")?;

    println!("{cfg:?}");
    for chan in &cfg.channels {
        download_channel(chan, &cfg.root_dir_path, &cfg.video_dir_name, &cfg.ytdlp_config_path)?;
        link_channel_playlists(chan, &cfg.root_dir_path, &cfg.video_dir_name)?;
    }

    // download_playlists();
    // link_channel_playlists()

    Ok(())
}
