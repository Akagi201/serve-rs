use std::{fs, path::PathBuf, time::SystemTime};

use axum::{
  body::Body,
  extract::{Path, State},
  http::StatusCode,
  response::{Html, IntoResponse, Response},
};
use percent_encoding::percent_decode;
use rinja::Template;
use tracing::{error, info};

#[derive(Template)]
#[template(path = "directory_listing.html", print = "code")]
struct DirListTemplate {
  current_path: String,
  parent_path: String,
  entries: Vec<DirEntry>,
}

struct DirEntry {
  name: String,
  is_dir: bool,
  size: String,
  modified: String,
  extension: String,
}

impl IntoResponse for DirListTemplate {
  fn into_response(self) -> Response<Body> {
    match self.render() {
      Ok(html) => Html(html).into_response(),
      Err(e) => Html(format!("Error rendering template: {e}")).into_response(),
    }
  }
}

#[derive(Clone)]
pub struct AppState {
  pub root_dir: PathBuf,
  pub hide_dotfiles: bool,
}

pub async fn render_index_root(State(state): State<AppState>) -> impl IntoResponse {
  render_index_internal(state, "".to_string()).await
}

pub async fn handle_directory_or_fallback(
  State(state): State<AppState>,
  Path(path): Path<String>,
) -> impl IntoResponse {
  let base_path = &state.root_dir;
  let decoded_path = match percent_decode(path.as_ref()).decode_utf8() {
    Ok(path) => path.to_string(),
    Err(_) => return (StatusCode::NOT_FOUND, "").into_response(),
  };

  let target_path = base_path.join(&decoded_path);

  // Security check
  if !target_path.starts_with(base_path) {
    return (StatusCode::NOT_FOUND, "").into_response();
  }

  // If it's a directory, render the directory listing
  if target_path.exists() && target_path.is_dir() {
    return render_index_internal(state, decoded_path).await.into_response();
  }

  // Otherwise, return NOT_FOUND to let the fallback serve the file
  (StatusCode::NOT_FOUND, "").into_response()
}

async fn render_index_internal(state: AppState, req_path: String) -> impl IntoResponse {
  let base_path = &state.root_dir;

  let req_path = match percent_decode(req_path.as_ref()).decode_utf8() {
    Ok(path) => path.to_string(),
    Err(e) => {
      error!("Failed to decode URL path: {}", e);
      return Html("Invalid URL encoding".to_string()).into_response();
    }
  };

  let target_path = base_path.join(&req_path);
  info!("base_path: {:?}, target_path: {:?}", base_path, target_path);

  // Security check: ensure target path is within base path
  if !target_path.starts_with(base_path) {
    error!("Path traversal attempt: {:?}", target_path);
    return Html("Access denied: Invalid path".to_string()).into_response();
  }

  // Check if target path exists
  if !target_path.exists() {
    return Html("Path not found".to_string()).into_response();
  }

  // If the path is a file, we should let the static file server handle it
  // by returning a response that will cause fallback
  if target_path.is_file() {
    // Return a NOT_FOUND status to trigger fallback to ServeDir
    use axum::http::StatusCode;
    return (StatusCode::NOT_FOUND, "").into_response();
  }

  // If it's not a directory at this point, return error
  if !target_path.is_dir() {
    return Html("Path is not a directory".to_string()).into_response();
  }

  let entries = match fs::read_dir(&target_path) {
    Ok(entries) => entries,
    Err(err) => {
      error!("Error reading directory {:?}: {}", target_path, err);
      return Html(format!("Error reading directory: {err}")).into_response();
    }
  };

  let mut dir_entries = Vec::new();
  for entry in entries {
    match entry {
      Ok(entry) => {
        let file_name = match entry.file_name().into_string() {
          Ok(name) => name,
          Err(_) => {
            error!("Invalid file name encoding");
            continue;
          }
        };

        match entry.file_type() {
          Ok(file_type) => {
            let is_dir = file_type.is_dir();

            // Skip dotfiles if hide_dotfiles is enabled
            if state.hide_dotfiles && file_name.starts_with('.') {
              continue;
            }

            let mut size = String::new();
            let mut modified = String::new();
            let extension = if !is_dir {
              entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
            } else {
              String::new()
            };

            // Get file metadata
            if let Ok(metadata) = entry.metadata() {
              if !is_dir {
                size = format_file_size(metadata.len());
              }

              if let Ok(time) = metadata.modified()
                && let Ok(duration) = time.duration_since(SystemTime::UNIX_EPOCH)
              {
                // Format as readable time
                let timestamp = duration.as_secs();
                modified = format_timestamp(timestamp);
              }
            }

            dir_entries.push(DirEntry { name: file_name, is_dir, size, modified, extension });
          }
          Err(e) => error!("Error getting file type for {:?}: {}", entry.path(), e),
        }
      }
      Err(e) => error!("Error processing directory entry: {}", e),
    }
  }

  // Sort entries: directories first, then files, both alphabetically
  dir_entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
    (true, false) => std::cmp::Ordering::Less,
    (false, true) => std::cmp::Ordering::Greater,
    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
  });

  // Calculate parent path as relative to base_path
  let parent_path = if target_path != *base_path {
    if let Some(parent) = target_path.parent() {
      if parent >= *base_path {
        // Convert to relative path from base_path
        match parent.strip_prefix(base_path) {
          Ok(relative) => {
            let path_str = relative.to_string_lossy();
            if path_str.is_empty() { "/".to_string() } else { format!("/{path_str}") }
          }
          Err(_) => String::new(),
        }
      } else {
        String::new()
      }
    } else {
      String::new()
    }
  } else {
    String::new()
  };

  // Current path relative to base_path
  let current_relative_path = match target_path.strip_prefix(base_path) {
    Ok(relative) => {
      let path_str = relative.to_string_lossy();
      if path_str.is_empty() { "/".to_string() } else { format!("/{path_str}") }
    }
    Err(_) => "/".to_string(),
  };

  let dir_list =
    DirListTemplate { current_path: current_relative_path, parent_path, entries: dir_entries };
  dir_list.into_response()
}

fn format_timestamp(timestamp: u64) -> String {
  // Simple timestamp formatting - you might want to use chrono for better formatting
  let days_ago = (std::time::SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs()
    - timestamp)
    / 86400;

  if days_ago == 0 {
    "Today".to_string()
  } else if days_ago == 1 {
    "Yesterday".to_string()
  } else if days_ago < 7 {
    format!("{days_ago} days ago")
  } else if days_ago < 30 {
    format!("{} weeks ago", days_ago / 7)
  } else {
    format!("{} months ago", days_ago / 30)
  }
}

fn format_file_size(size: u64) -> String {
  const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
  let mut size = size as f64;
  let mut unit_index = 0;

  while size >= 1024.0 && unit_index < UNITS.len() - 1 {
    size /= 1024.0;
    unit_index += 1;
  }

  if unit_index == 0 {
    format!("{:.0} {}", size, UNITS[unit_index])
  } else {
    format!("{:.1} {}", size, UNITS[unit_index])
  }
}
