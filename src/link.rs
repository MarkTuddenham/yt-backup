use anyhow::Result;
use std::{fs, process::Command};

use crate::config::{Channel, Playlist};

pub fn link_channel_playlists(
    chan: &Channel,
    root_dir_path: &str,
    video_dir_name: &str,
) -> Result<()> {
    let video_dir_path = format!("{root_dir_path}/{}/{video_dir_name}", chan.name);
    let playlists = get_playlists_in_channel(chan)?;
    println!("Channel '{}' has these playlists: {playlists:?}", chan.name);

    for pl in playlists {
        let playlist_dir_path = format!("{root_dir_path}/{}/{}", chan.name, pl.name);
        fs::create_dir_all(&playlist_dir_path)?;
        link_playlist(&pl, &playlist_dir_path, &video_dir_path)?;
    }
    Ok(())
}

fn get_playlists_in_channel(chan: &Channel) -> Result<Vec<Playlist>> {
    if let Some(url) = &chan.url {
        let url = url.to_owned() + "/playlists";
        let output = Command::new("yt-dlp")
            .args(["--flat-playlist", "--get-filename", &url])
            .output()?
            .stdout;

        let output = std::str::from_utf8(&output)?;

        Ok(output
            .split('\n')
            .filter_map(|s| s.try_into().ok())
            .collect())
    } else {
        Err(anyhow::Error::msg("No channel url"))
    }
}

fn get_playlist_video_names(playlist: &Playlist) -> Result<Vec<String>> {
    let output = Command::new("yt-dlp")
        .args([
            "--flat-playlist",
            "--get-filename",
            "-o",
            "%(title)s",
            &playlist.id,
        ])
        .output()?
        .stdout;

    let output = std::str::from_utf8(&output)?;

    Ok(output
        .split('\n')
        .map(|s| s.to_owned())
        .filter(|s| !s.is_empty())
        .collect())
}

fn link_playlist(playlist: &Playlist, playlist_dir_path: &str, video_dir_path: &str) -> Result<()> {
    let pl_video_names = get_playlist_video_names(playlist)?;
    println!(
        "Playlist '{}' has these videos: {pl_video_names:?}",
        playlist.name
    );

    for (i, video_name) in pl_video_names.iter().enumerate() {
        let original_file_path = format!("{video_dir_path}/{video_name}");
        let new_file_path = format!("{playlist_dir_path}/{} - {video_name}", i + 1);

        let extensions = get_extensions_for_existing_video_files(video_dir_path, video_name)?;
        // println!("Found files with extensions: {extensions:?}");

        for ext in &extensions {
            let original_file = format!("{original_file_path}.{ext}");
            let new_file = format!("{new_file_path}.{ext}");
            // println!("Linking {new_file}->{original_file}");
            fs::hard_link(original_file, new_file)?;
        }
    }

    Ok(())
}
fn get_extensions_for_existing_video_files(
    video_dir_path: &str,
    video_name: &str,
) -> Result<Vec<String>> {
    let files = fs::read_dir(video_dir_path)?
        .filter_map(|entry| match entry {
            Ok(entry) => {
                let pb = entry.path();
                if let Some(stem) = pb.file_stem() {
                    if stem == video_name {
                        return pb
                            .extension()
                            .and_then(|s| s.to_os_string().into_string().ok());
                    }
                }

                None
            }
            Err(_) => None,
        })
        .collect();

    Ok(files)
}
