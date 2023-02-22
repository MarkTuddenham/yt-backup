use anyhow::Ok;
use clap::Parser;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, Registry};

mod config;
use crate::config::load_config;
mod download;
use crate::download::{download_channel, set_last_download_date};
mod link;
use crate::link::link_channel_playlists;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Path to config file, if unspecified then \"./config.toml\" or \"<config_dir>/yt-backup/config.toml\" is used"
    )]
    config: Option<String>,

    #[arg(short, long, help = "Remove current playlist links and relink")]
    relink: bool,

    #[arg(short, long, help = "Do not download new videos")]
    skip_download: bool,

    #[arg(
        short,
        long,
        help = "Only download videos released after this command was last run"
    )]
    incremental_download: bool,

    #[arg(
        short,
        long,
        default_value_t = tracing::Level::INFO,
        help = "stdout log level (trace, debug, info, warn, error)"
    )]
    log_level: tracing::Level,
}

fn main() {
    let args = Args::parse();

    let (file_writer, _guard) = dirs::cache_dir()
        .map(|p| p.join("yt-backup"))
        .map(|p| tracing_appender::rolling::daily(p, "log"))
        .map(tracing_appender::non_blocking)
        .unzip();

    let file_logger =
        file_writer.map(|file_writer| fmt::layer().with_ansi(false).with_writer(file_writer));

    let stdout_logger = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(LevelFilter::from_level(args.log_level));

    let subscriber = Registry::default().with(file_logger).with(stdout_logger);

    tracing::subscriber::set_global_default(subscriber).expect("unable to set global tracing subscriber");

    if let Err(e) = app(&args) {
        tracing::error!("{e}");
    }
}
fn app(args: &Args) -> anyhow::Result<()> {
    let cfg = load_config(args.config.as_ref())?;

    tracing::info!("{cfg:?}");
    let version = check_ytdlp_version()?;
    tracing::info!("Using yt-dlp version {version}");

    for chan in &cfg.channels {
        tracing::trace!("{chan:?}");
        if !args.skip_download {
            let res = download_channel(
                chan,
                &cfg.root_dir_path,
                &cfg.video_dir_name,
                &cfg.ytdlp_config_path,
                args.incremental_download,
            );

            if let Err(e) = res {
                tracing::error!("Error downloading channel {}: {}", chan.name, e);
            }
        }
        link_channel_playlists(
            chan,
            &cfg.root_dir_path,
            &cfg.video_dir_name,
            &cfg.link_type,
            args.relink,
        )?;
    }

    set_last_download_date()?;

    // if !args.skip_download{
    // download_playlists();
    // }
    // link_playlists()

    Ok(())
}

fn check_ytdlp_version() -> anyhow::Result<String> {
    std::process::Command::new("yt-dlp")
        .arg("--version")
        .output()
        .map_err(|_| anyhow::Error::msg("Could not find a yt-dlp executable"))
        .and_then(|o| {
            std::str::from_utf8(&o.stdout)
                .map(|s| s.to_owned())
                .map_err(|_| anyhow::Error::msg("Could not convert yt-dlp version to string"))
        })
}
