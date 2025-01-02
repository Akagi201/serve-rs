use axum::{routing::get, Router};
use clap::Parser;
use config::Cli;
use eyre::Result;
use handlers::{render_index, AppState};
use log::init_log;
use shadow_rs::shadow;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;

mod config;
mod handlers;
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
    .not_found_service(get(render_index).with_state(AppState { root_dir: cli.root_dir.clone() }));
  let app = Router::new().fallback_service(service).layer(TraceLayer::new_for_http());
  let listener = TcpListener::bind(&cli.addr).await?;

  info!("Listening on: {}", listener.local_addr()?);

  axum::serve(listener, app).await?;

  Ok(())
}
