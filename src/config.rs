use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Cli {
  /// The root directory
  #[clap(short, long, default_value = "./")]
  pub root_dir: PathBuf,
  /// The address to bind to
  #[clap(short, long, default_value = "0.0.0.0:8327")]
  pub addr: String,
  /// Print the version
  #[clap(short, long, default_value = "false")]
  pub version: bool,
}
