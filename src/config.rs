use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(name = "serve-rs")]
#[command(about = "A minimal static file server with directory browsing")]
#[command(disable_version_flag = true)]
pub struct Cli {
  /// The root directory to serve files from
  #[clap(short, long, default_value = "./")]
  pub root_dir: PathBuf,

  /// The address to bind to
  #[clap(short, long, default_value = "0.0.0.0:8327")]
  pub addr: String,

  /// Print the version
  #[clap(short, long, default_value = "false")]
  pub version: bool,

  /// Log level (trace, debug, info, warn, error)
  #[clap(short, long, default_value = "info")]
  pub log_level: String,

  /// Hide dotfiles in directory listings
  #[clap(long, default_value = "false")]
  pub hide_dotfiles: bool,

  /// Enable CORS headers
  #[clap(long, default_value = "false")]
  pub enable_cors: bool,
}
