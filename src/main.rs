mod cache;
mod config;
mod platform;
mod steam;
mod tui;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "steam-proton-browser", about = "TUI for browsing Steam & Proton folders")]
struct Cli {
    /// Override Steam installation path
    #[arg(long)]
    steam_path: Option<String>,

    /// Force a full rescan, ignoring cached data
    #[arg(long)]
    refresh: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config_path = config::settings::config_file_path();
    let settings = config::persistence::load(&config_path);

    let mut steam_roots = if let Some(ref path) = cli.steam_path {
        vec![std::path::PathBuf::from(path)]
    } else {
        steam::discovery::discover_steam_roots()
    };

    for extra in &settings.extra_steam_paths {
        if extra.is_dir() && !steam_roots.contains(extra) {
            steam_roots.push(extra.clone());
        }
    }

    if steam_roots.is_empty() {
        anyhow::bail!("No Steam installation found. Use --steam-path to specify one manually.");
    }

    let cache_path = cache::cache_file_path();
    let library = if !cli.refresh {
        cache::load(&cache_path).and_then(|c| {
            if cache::is_valid(&c, &steam_roots) {
                Some(c.into_library())
            } else {
                None
            }
        })
    } else {
        None
    };

    let library = match library {
        Some(lib) => lib,
        None => {
            let lib = steam::library::scan_libraries(&steam_roots)?;
            let _ = cache::save(&cache_path, &lib, &steam_roots);
            lib
        }
    };

    tui::app::run(library)?;

    Ok(())
}
