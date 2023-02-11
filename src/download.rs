use crate::config::Channel;
use anyhow::Result;
use std::{
    fs,
    process::{Command, Output},
};

pub fn download_channel(
    chan: &Channel,
    root_dir_path: &str,
    video_dir: &str,
    ytdlp_config_path: &str,
) -> Result<Output> {
    let folder = format!("{root_dir_path}/{}/{video_dir}", chan.name);
    fs::create_dir_all(&folder)?;

    if let Some(url) = &chan.url {
        println!("Downloading {} to {folder}", chan.name);

        //todo: args -> " --dateafter " + last_download_time
        return Command::new("yt-dlp")
            .args([
                "--config",
                ytdlp_config_path,
                "-o",
                "%(title)s.%(ext)s",
                url,
            ])
            .current_dir(folder)
            .output()
            .map_err(anyhow::Error::msg);
    } else {
        Err(anyhow::Error::msg("No url to download from"))
    }
}
