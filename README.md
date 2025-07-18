# serve-rs

A minimalist static file server with elegant directory browsing and Apple-inspired UI.

## Features

- 🚀 **Fast & Lightweight** - Built with Rust and Axum
- 📁 **Directory Browsing** - Beautiful file listings with search and sorting
- 🎨 **Modern UI** - Apple-inspired design with dark/light theme support
- 📦 **Direct Downloads** - Click files to download instantly
- 🔒 **Secure** - Path traversal protection built-in
- ⚙️ **Configurable** - Custom port, CORS, dotfile visibility

## Install

```sh
cargo install --git https://github.com/Akagi201/serve-rs
```

## Usage

### Basic Usage

```sh
# Serve current directory on default port (8327)
serve

# Serve specific directory
serve --root-dir /path/to/files

# Custom address and port
serve --addr 127.0.0.1:3000
```

### Options

```sh
serve [OPTIONS]

Options:
  -r, --root-dir <PATH>     Directory to serve [default: ./]
  -a, --addr <ADDR>         Address to bind [default: 0.0.0.0:8327]
  -l, --log-level <LEVEL>   Log level [default: info]
      --hide-dotfiles       Hide dotfiles in listings
      --enable-cors         Enable CORS headers
  -v, --version             Print version
  -h, --help                Print help
```

## Examples

```sh
# Share current folder
serve

# Share on localhost only
serve --addr 127.0.0.1:8080

# Hide dotfiles and enable CORS
serve --hide-dotfiles --enable-cors

# Serve with debug logging
serve --log-level debug
```
