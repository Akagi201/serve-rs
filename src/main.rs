use axum::{Router, routing::get};
use clap::Parser;
use config::Cli;
use eyre::Result;
use handlers::{AppState, handle_file_or_directory, health_check, render_index_root, serve_file};
use log::init_log;
use shadow_rs::shadow;
use tokio::net::TcpListener;
use tower_http::{
  cors::{Any, CorsLayer},
  services::ServeDir,
};
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

  init_log(&cli.log_level)?;
  info!("cli: {:?}", cli);

  let app_state = AppState { root_dir: cli.root_dir.clone(), hide_dotfiles: cli.hide_dotfiles };

  // Create router with directory listing handler first, then static file serving
  let serve_dir = ServeDir::new(&app_state.root_dir);

  let mut app = Router::new()
    .route("/", get(render_index_root))
    .route("/health", get(health_check))
    .route("/download/{*path}", get(serve_file))
    .route("/{*path}", get(handle_file_or_directory))
    .fallback_service(serve_dir)
    .with_state(app_state);

  // Add CORS layer if enabled
  if cli.enable_cors {
    app = app.layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));
  }

  let listener = TcpListener::bind(&cli.addr).await?;
  info!("Listening on: {}", listener.local_addr()?);

  axum::serve(listener, app).await?;

  Ok(())
}
