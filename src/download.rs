use crate::config::Channel;
use anyhow::Result;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

pub fn download_channel(
    chan: &Channel,
    root_dir_path: impl AsRef<Path>,
    video_dir: &str,
    ytdlp_config_path: impl AsRef<Path>,
    incremental_download: bool,
) -> Result<Output> {
    let folder = root_dir_path.as_ref().join(&chan.name).join(video_dir);
    fs::create_dir_all(&folder)?;

    let download_after = get_last_download_date();

    if let Some(url) = &chan.url {
        tracing::info!("Downloading {} to {}", chan.name, folder.display());
        let ytdlp_config_path = ytdlp_config_path
            .as_ref()
            .canonicalize()?
            .into_os_string()
            .into_string()
            .map_err(|_| {
                anyhow::Error::msg("Could not convert yt-dlp config path to utf-8 string")
            })?;

        let mut args = vec![
            "--config".into(),
            ytdlp_config_path,
            "-o".into(),
            "%(title)s.%(ext)s".into(),
        ];

        if let (true, Some(download_after)) = (incremental_download, download_after) {
            args.push("--dateafter".into());
            args.push(download_after);
        }

        args.push(url.to_owned());
        tracing::trace!("Running \"yt-dlp {args:?}\" in {folder:?}");

        return Command::new("yt-dlp")
            .args(args)
            .current_dir(folder)
            .output()
            .map_err(anyhow::Error::msg);
    } else {
        Err(anyhow::Error::msg("No url to download from"))
    }
}

pub fn set_last_download_date() -> io::Result<()> {
    if let Some(path) = get_last_download_date_file_path() {
        let ts = chrono::offset::Utc::now();
        if let (false, Some(parent)) = (path.exists(), path.parent()) {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, ts.format("%Y%m%d").to_string())?;
    }

    Ok(())
}
fn get_last_download_date() -> Option<String> {
    if let Some(path) = get_last_download_date_file_path() {
        return fs::read_to_string(path).ok();
    }

    None
}

fn get_last_download_date_file_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("yt-backup").join("last_download_date"))
}
