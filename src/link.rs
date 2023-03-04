use anyhow::Result;
use std::{fs, io, path::Path, process::Command};

use crate::config::{Channel, LinkType, Playlist};

pub fn link_channel_playlists(
    chan: &Channel,
    root_dir_path: impl AsRef<Path>,
    video_dir_name: &str,
    link_type: &LinkType,
    relink: bool,
) -> Result<()> {
    tracing::trace!("Linking {}", chan.name);
    let video_dir_path = root_dir_path.as_ref().join(&chan.name).join(video_dir_name);
    let playlists = get_playlists_in_channel(chan)?;
    tracing::info!("Channel '{}' has these playlists: {playlists:?}", chan.name);

    for pl in playlists {
        let playlist_dir_path = root_dir_path.as_ref().join(&chan.name).join(&pl.name);
        if relink {
            fs::remove_dir_all(&playlist_dir_path)?;
        }
        fs::create_dir_all(&playlist_dir_path)?;
        link_playlist(&pl, &playlist_dir_path, &video_dir_path, link_type)?;
    }
    Ok(())
}

fn get_playlists_in_channel(chan: &Channel) -> Result<Vec<Playlist>> {
    if let Some(url) = &chan.url {
        // view=1 is created playlists only, not recommended.
        let url = url.to_owned() + "/playlists?view=1";
        tracing::trace!("get_playlists_in_channel: {url}");
        let output = Command::new("yt-dlp")
            .args(["--flat-playlist", "--get-filename", &url])
            .output()?
            .stdout;

        let output = std::str::from_utf8(&output)?;
        tracing::trace!("get_playlists_in_channel raw output: {output}");

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
        .filter(|s| s != "[Private video]" && s != "[Deleted video]")
        .collect())
}

fn link_playlist(
    playlist: &Playlist,
    playlist_dir_path: impl AsRef<Path>,
    video_dir_path: impl AsRef<Path>,
    link_type: &LinkType,
) -> Result<()> {
    let pl_video_names = get_playlist_video_names(playlist)?;
    tracing::info!(
        "Playlist '{}' has these videos: {pl_video_names:?}",
        playlist.name
    );

    for (i, video_name) in pl_video_names.iter().enumerate() {
        // .set_extension will override any "." in a video name the first time it's set,
        // so set a temporary extension to add an extra ".":
        // "video name..." -> "video_name....tmp"
        // "video.name" -> "video.name.tmp"
        let mut original_file_path = video_dir_path.as_ref().join(video_name.to_owned() + ".tmp");
        let mut new_file_path =
            playlist_dir_path
                .as_ref()
                .join(format!("{:0>3} - {}", i + 1, video_name));

        let extensions =
            get_extensions_for_existing_video_files(video_dir_path.as_ref(), video_name)?;

        if extensions.is_empty() {
            tracing::warn!(
                "Could not find any videos matching \"{video_name}\" in \"{}\"",
                video_dir_path.as_ref().display()
            )
        }

        for ext in &extensions {
            original_file_path.set_extension(ext);
            new_file_path.set_extension(ext);
            tracing::trace!(
                "Linking: {}->{}",
                new_file_path.display(),
                original_file_path.display()
            );

            //TODO: what if the other type of link already exits? how do we tell if it is a hard link?
            let link_res: io::Result<()> = match link_type {
                LinkType::Hard => fs::hard_link(&original_file_path, &new_file_path),
                LinkType::Soft => make_symlink(&original_file_path, &new_file_path),
            };

            match link_res {
                Ok(()) => (),
                Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(e) => return Err(anyhow::Error::msg(e)),
            }
        }
    }

    Ok(())
}

fn make_symlink(file_path: impl AsRef<Path>, link_path: impl AsRef<Path>) -> io::Result<()> {
    #[cfg(unix)]
    return std::os::unix::fs::symlink(file_path, link_path);

    #[cfg(windows)]
    return std::os::windows::fs::symlink_file(file_path, link_path);
}

fn get_extensions_for_existing_video_files(
    video_dir_path: impl AsRef<Path>,
    video_name: &str,
) -> Result<Vec<String>> {
    let extensions = fs::read_dir(video_dir_path)?
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

    Ok(extensions)
}
