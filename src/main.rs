use axum::Router;
use clap::Parser;
use config::Cli;
use eyre::Result;
use log::init_log;
use shadow_rs::shadow;
use tokio::net::TcpListener;
use tower_http::{
  services::{ServeDir, ServeFile},
  trace::TraceLayer,
};
use tracing::info;

mod config;
mod log;

shadow!(build);

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();
  if cli.version {
    println!("{}", build::VERSION);
    return Ok(());
  }
  init_log("info")?;
  info!("cli: {:?}", cli);

  let service = ServeDir::new(cli.root_dir.clone())
    .not_found_service(ServeFile::new(cli.root_dir.join("index.html")));
  let app = Router::new().fallback_service(service).layer(TraceLayer::new_for_http());
  let listener = TcpListener::bind(&cli.addr).await?;

  info!("Listening on: {}", listener.local_addr()?);

  axum::serve(listener, app).await?;

  Ok(())
}
