# Copilot Instructions for serve-rs

## Project Overview
This is a minimalist static file server built with Rust and Axum. The core architecture follows a single-binary approach serving files with directory browsing fallback when files aren't found.

## Architecture Patterns

### Main Components
- **`main.rs`**: Entry point using Axum router with fallback service pattern
- **`config.rs`**: CLI configuration using clap derive macros
- **`handlers.rs`**: Request handlers with custom directory listing logic
- **`log.rs`**: Centralized logging setup with tracing-subscriber

### Key Design Decisions
- Uses Axum's `ServeDir` service for static file serving with custom `not_found_service` for directory listings
- Template rendering via Rinja (formerly Askama) with HTML templates in `templates/`
- Security: Path traversal protection in `render_index` handler using `starts_with(base_path)` check

## Development Workflows

### Build & Development
```bash
# Use Just for common tasks
just format    # Format code with taplo + cargo fmt
just lint      # Full linting pipeline with clippy rules
just test      # Run test suite

# Direct cargo commands
cargo run -- --root-dir /path/to/serve --addr 127.0.0.1:3000
cargo build --release
```

### Code Quality Standards
- Uses `cargo +nightly` for formatting/linting (see Justfile)
- Strict clippy rules: `-D warnings -A clippy::derive_partial_eq_without_eq -D clippy::unwrap_used`
- Template code generation via Rinja's `#[template(print = "code")]` for debugging

## Project-Specific Conventions

### Error Handling
- Uses `eyre::Result` for error propagation throughout the codebase
- Template rendering errors return HTML error responses rather than panicking
- Directory access errors use `eprintln!` for server-side logging + HTML user feedback

### CLI Design Pattern
```rust
#[derive(Clone, Debug, Parser)]
pub struct Cli {
  #[clap(short, long, default_value = "./")]
  pub root_dir: PathBuf,
  // Version flag returns early without starting server
}
```

### State Management
- Uses Axum's `State` extractor with `AppState` struct containing only `root_dir`
- No database or persistent state - purely stateless request handling

### Template Architecture
- Templates in `templates/` directory using Rinja syntax
- `DirListTemplate` implements `IntoResponse` directly for type-safe HTML rendering
- URL encoding handled via template filters: `{{ path|urlencode }}`

## Integration Points

### External Dependencies
- **axum**: Web framework with tower-http for static file serving
- **rinja**: Template engine (successor to Askama)
- **shadow-rs**: Build-time metadata injection (see `build.rs`)
- **percent-encoding**: URL decoding for file paths
- **clap**: CLI argument parsing with derive macros

### Build System
- `build.rs` uses shadow-rs for version injection
- Edition 2024 Rust with full tokio features
- No complex build pipeline - single binary output

## Key Files to Understand
- `src/handlers.rs`: Core request routing and directory listing logic
- `templates/index.html`: Directory browsing UI with Tailwind CSS
- `Justfile`: Development workflow automation
- `Cargo.toml`: Dependency management and build configuration
