use clap::Parser;
use std::path::PathBuf;

/// duplff - find and remove duplicate files
#[derive(Parser, Debug)]
#[command(name = "duplff", version, about)]
pub struct Cli {
    /// Directories to scan for duplicates
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// File extensions to include (e.g. py rs js)
    #[arg(short = 'e', long = "ext")]
    pub extensions: Option<Vec<String>>,

    /// Minimum file size in bytes (default: 1)
    #[arg(short = 'm', long = "min-size", default_value = "1")]
    pub min_size: u64,

    /// Maximum file size in bytes
    #[arg(short = 'M', long = "max-size")]
    pub max_size: Option<u64>,

    /// Priority directories (files here are preferred to keep)
    #[arg(short = 'p', long = "priority")]
    pub priority: Option<Vec<PathBuf>>,

    /// Exclude directories/patterns (glob, repeatable)
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Option<Vec<String>>,

    /// Follow symbolic links
    #[arg(short = 'L', long = "follow-symlinks")]
    pub follow_symlinks: bool,

    /// Output JSON report (non-interactive)
    #[arg(long)]
    pub json: bool,

    /// Show deletion plan without deleting (non-interactive)
    #[arg(long)]
    pub dry_run: bool,

    /// Output CSV report (non-interactive)
    #[arg(long)]
    pub csv: bool,

    /// Disable hash cache
    #[arg(long)]
    pub no_cache: bool,

    /// Byte-by-byte verification after hash match
    #[arg(long)]
    pub paranoid: bool,
}

impl Cli {
    /// Convert CLI args into a duplff-core ScanConfig.
    pub fn to_scan_config(&self) -> duplff_core::models::ScanConfig {
        duplff_core::models::ScanConfig {
            roots: self.paths.clone(),
            extensions: self.extensions.clone(),
            min_size: self.min_size,
            max_size: self.max_size,
            priority_paths: self.priority.clone().unwrap_or_default(),
            follow_symlinks: self.follow_symlinks,
            exclude_patterns: self.exclude.clone().unwrap_or_default(),
            no_cache: self.no_cache,
            paranoid: self.paranoid,
        }
    }
}
