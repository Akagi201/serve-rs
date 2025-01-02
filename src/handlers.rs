use std::{fs, path::PathBuf};

use axum::{
  body::Body,
  extract::{Request, State},
  response::{Html, IntoResponse, Response},
};
use percent_encoding::percent_decode;
use rinja::Template;
use tracing::info;

#[derive(Template)]
#[template(path = "index.html", print = "code")]
struct DirListTemplate {
  current_path: String,
  parent_path: String,
  entries: Vec<DirEntry>,
}

struct DirEntry {
  name: String,
  is_dir: bool,
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
}

pub async fn render_index(State(state): State<AppState>, req: Request<Body>) -> impl IntoResponse {
  let base_path = &state.root_dir;
  let req_path = req.uri().path();
  let req_path = req_path.trim_start_matches('/');
  let req_path = percent_decode(req_path.as_ref()).decode_utf8_lossy().to_string();
  let target_path = base_path.join(req_path);
  info!("base_path: {:?}, target_path: {:?}", base_path, target_path);

  if !target_path.starts_with(base_path) {
    return Html(format!("Invalid path: {target_path:?}")).into_response();
  }

  let entries = match fs::read_dir(&target_path) {
    Ok(entries) => entries,
    Err(err) => {
      eprintln!("Error reading directory: {err}");
      return Html(format!("Internal Server Error: {err}")).into_response();
    }
  };

  let mut dir_entries = Vec::new();
  for entry in entries {
    match entry {
      Ok(entry) => {
        let file_name = entry.file_name().into_string().unwrap_or_default();
        match entry.file_type() {
          Ok(file_type) => {
            dir_entries.push(DirEntry { name: file_name, is_dir: file_type.is_dir() });
          }
          Err(e) => eprintln!("Error getting file type: {e}"),
        }
      }
      Err(e) => eprintln!("Error processing directory entry: {e}"),
    }
  }

  let parent_path = if target_path != *base_path && target_path != PathBuf::from("/") {
    target_path.parent().expect("target path parent folder").display().to_string()
  } else {
    "".to_string()
  };

  let dir_list = DirListTemplate {
    current_path: target_path.display().to_string(),
    parent_path,
    entries: dir_entries,
  };
  dir_list.into_response()
}
