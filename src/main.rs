mod cache;
mod config;
mod platform;
mod steam;
mod tui;

struct Cli {
    steam_path: Option<String>,
    refresh: bool,
}

#[derive(Debug)]
enum ParseArgsError {
    UnknownArg(String),
    ShowHelp,
}

fn try_parse_args(args: impl Iterator<Item = String>) -> Result<Cli, ParseArgsError> {
    let mut cli = Cli {
        steam_path: None,
        refresh: false,
    };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--steam-path" => {
                cli.steam_path = args.next();
            }
            "--refresh" => {
                cli.refresh = true;
            }
            "--help" | "-h" => {
                return Err(ParseArgsError::ShowHelp);
            }
            other => {
                return Err(ParseArgsError::UnknownArg(other.to_string()));
            }
        }
    }
    Ok(cli)
}

fn parse_args() -> Cli {
    match try_parse_args(std::env::args().skip(1)) {
        Ok(cli) => cli,
        Err(ParseArgsError::ShowHelp) => {
            eprintln!("steam-proton-browser — TUI for browsing Steam & Proton folders\n");
            eprintln!("Usage: steam-proton-browser [OPTIONS]\n");
            eprintln!("Options:");
            eprintln!("  --steam-path <PATH>  Override Steam installation path");
            eprintln!("  --refresh            Force a full rescan, ignoring cached data");
            eprintln!("  -h, --help           Print this help message");
            std::process::exit(0);
        }
        Err(ParseArgsError::UnknownArg(arg)) => {
            eprintln!("Unknown argument: {arg}");
            eprintln!("Run with --help for usage.");
            std::process::exit(1);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_args();

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
        return Err("No Steam installation found. Use --steam-path to specify one manually.".into());
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args<'a>(args: &'a [&'a str]) -> impl Iterator<Item = String> + 'a {
        args.iter().map(|s| s.to_string())
    }

    #[test]
    fn test_no_args() {
        let cli = try_parse_args(args(&[])).unwrap();
        assert_eq!(cli.steam_path, None);
        assert!(!cli.refresh);
    }

    #[test]
    fn test_steam_path() {
        let cli = try_parse_args(args(&["--steam-path", "/opt/steam"])).unwrap();
        assert_eq!(cli.steam_path.as_deref(), Some("/opt/steam"));
        assert!(!cli.refresh);
    }

    #[test]
    fn test_refresh() {
        let cli = try_parse_args(args(&["--refresh"])).unwrap();
        assert!(cli.refresh);
    }

    #[test]
    fn test_combined_args() {
        let cli = try_parse_args(args(&["--steam-path", "/games", "--refresh"])).unwrap();
        assert_eq!(cli.steam_path.as_deref(), Some("/games"));
        assert!(cli.refresh);
    }

    #[test]
    fn test_unknown_arg() {
        let result = try_parse_args(args(&["--bogus"]));
        assert!(matches!(result, Err(ParseArgsError::UnknownArg(_))));
    }

    #[test]
    fn test_help_flag() {
        let result = try_parse_args(args(&["--help"]));
        assert!(matches!(result, Err(ParseArgsError::ShowHelp)));
    }

    #[test]
    fn test_short_help_flag() {
        let result = try_parse_args(args(&["-h"]));
        assert!(matches!(result, Err(ParseArgsError::ShowHelp)));
    }
}
