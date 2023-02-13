use anyhow::Result;
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub root_dir_path: String,

    #[serde(default = "default_video_dir_name")]
    pub video_dir_name: String,

    #[serde(default = "default_link_type")]
    pub link_type: LinkType,

    // TODO: optional or just put all the config options in this config file
    pub ytdlp_config_path: PathBuf,

    pub channels: Vec<Channel>,
    // pub playlists: Vec<Playlist>,
}

fn default_video_dir_name() -> String {
    "_videos".into()
}

fn default_link_type() -> LinkType {
    LinkType::Hard
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkType {
    Hard,
    Soft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
}

fn get_config_path(config_path: Option<impl AsRef<Path>>) -> Result<PathBuf> {
    let config_paths = vec![
        config_path.map(|cp| cp.as_ref().to_owned()),
        Some(PathBuf::new().join("./config.toml")),
        config_dir().map(|path| path.join("yt-backup").join("config.toml")),
    ];

    for config_path in config_paths.into_iter().flatten() {
        if config_path.is_file() {
            return Ok(config_path);
        }
    }

    Err(anyhow::Error::msg("Could not find a valid config file"))
}

pub fn load_config(config_path: Option<impl AsRef<Path>>) -> Result<Config> {
    let config_path = get_config_path(config_path)?;


    let config_data = fs::read_to_string(config_path)?;
    let mut config: Config = toml::from_str(&config_data).map_err(anyhow::Error::msg)?;

    // TODO: check that this url exists
    config.channels.iter_mut().for_each(|c| {
        if c.url.is_none() {
            c.url = Some(format!("https://youtube.com/c/{}", c.name));
        }
    });

    Ok(config)
}

impl TryFrom<&str> for Playlist {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // e.g. Tuning Japanese Planes [PLSVL0S3stcMRpN9zl_PxEfBJvr7HCNd6Y].NA

        let len = value.len();
        if len < 41 {
            return Err(anyhow::Error::msg(
                "string is not long enough to be a playlist in the form \"<name> [<34-char id>].NA\"",
            ));
        }

        Ok(Playlist {
            id: value[len - 38..len - 4].to_owned(),
            name: value[..len - 40].to_owned(),
        })
    }
}

// last_download_time: 20211018
