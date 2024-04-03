<div align=center>
  <a href="https://github.com/marktuddenham/yt-backup">
    <img src="https://raw.githubusercontent.com/marktuddenham/yt-backup/master/images/yt_backup.svg" alt="yt-backup" width="200">
  </a>
  <h1>yt-backup</h1>

  ![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
  [![](https://img.shields.io/badge/docker-ghcr.io%2Fmarktuddenham%2Fyt--backup-blue)](https://github.com/MarkTuddenham/yt-backup/pkgs/container/yt-backup)

 **Backup your favourite YouTube channels using [yt-dlp](https://github.com/yt-dlp/yt-dlp)**

</div>



Install from crates.io
```bash
cargo install --locked yt-backup
```

Run with the default config location `<config_dir>/yt-backup/config.toml` or `./config.toml`
```bash
yt-backup
```
or specify a config path
```bash
yt-backup --config /path/to/config.toml
```

> [!CAUTION]
> Not for use in a way that violates YouTube's T&Cs.
> It's up to you to check your use is compliant.

## Configuration

Example configuration:

```toml
root_dir_path = "/path/to/backup/" # directory to download everything to (default "./")
link_type = "hard" # "hard" or "soft" use symlinks or hard links (default "hard")
ytdlp_config_path = "/path/to/configs/yt-dlp.config"

playlists = [ # download the below playlists but unassociated with a channel.
    "PLUeHTafWecAVblNx278wBxkIQXw7iJws3"
]

[[channels]]
name = "JapaneseToolsAustralia"
# url defaults to "https://youtube.com/c/<name>"
[[channels]]
name = "English Country Life"
url = "https://www.youtube.com/channel/UCGzRPk4-weg4odbYNCjujJA"
```

Example yt-dlp configuration:

```
-f "bestvideo[height>=720]+bestaudio/best"
-ciw
--all-subs
--embed-subs
--no-progress
--no-colors
--write-thumbnail
--write-description
-r 3M
```

## Docker
```bash
docker run \
    -v $(pwd)/path/to/video/store:/app/data \
    -v $(pwd)/yt-dlp.config:/app/yt-dlp.config \
    -v $(pwd)/config.toml:/app/config.toml \
    --name yt-backup \
    ghcr.io/marktuddenham/yt-backup:latest
```
with config

```toml
root_dir_path = "/app/data/"
ytdlp_config_path = "/app/yt-dlp.config"

```
