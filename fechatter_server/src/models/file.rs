use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use crate::{AppError, models::ChatFile};
use tracing::{info, warn};

use sha1::{Digest, Sha1};

impl ChatFile {
  pub fn new(ws_id: i64, filename: &str, data: &[u8]) -> Self {
    let sha1_file_hash = Sha1::digest(data);
    let hash = hex::encode(sha1_file_hash);
    let ext = filename
      .rsplit_once('.')
      .unwrap_or(("", "txt"))
      .1
      .to_string();

    Self {
      workspace_id: ws_id,
      ext,
      hash,
    }
  }

  pub fn url(&self) -> String {
    format!("/files/{}", self.hash_to_path())
  }

  /// split hash into 3 parts, first 2 with 3 chars, last with 2 chars
  /// For example: "1/943/a70/2d06f34599aee1f8da8ef9f7296031d699.txt",
  pub fn hash_to_path(&self) -> String {
    let (part1, part2) = self.hash.split_at(3);
    let (part2, part3) = part2.split_at(3);

    format!(
      "{}/{}/{}/{}.{}",
      self.workspace_id, part1, part2, part3, self.ext
    )
  }

  pub fn from_path<P: AsRef<Path>>(&self, base_dir: P) -> PathBuf {
    let path = base_dir.as_ref().join(self.hash_to_path());
    path
  }
}

impl FromStr for ChatFile {
  type Err = AppError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    info!("[TRACE] ChatFile::from_str parsing: {}", s);
    let Some(s) = s.strip_prefix("/files/") else {
      warn!("[TRACE] Invalid path prefix: {}", s);
      return Err(AppError::ChatFileError(
        "Invaild chat file path".to_string(),
      ));
    };
    info!("[TRACE] Stripped prefix, remaining: {}", s);

    let parts: Vec<&str> = s.split('/').collect();
    info!("[TRACE] Split parts: {:?}", parts);
    info!("[TRACE] Parts length: {}", parts.len());
    info!("[TRACE] Expected format: /files/<workspace_id>/<part1>/<part2>/<part3>.<ext>");

    if parts.len() != 4 {
      warn!("[TRACE] Invalid parts length: {}", parts.len());
      warn!("[TRACE] Expected format: /files/<workspace_id>/<part1>/<part2>/<part3>.<ext>");
      return Err(AppError::ChatFileError(
        "Invalid chat file path".to_string(),
      ));
    }

    let Ok(ws_id) = parts[0].parse::<i64>() else {
      warn!("[TRACE] Invalid workspace id: {}", parts[0]);
      return Err(AppError::ChatFileError(format!(
        "Invalid workspace id: {}",
        parts[0]
      )));
    };
    info!("[TRACE] Parsed workspace id: {}", ws_id);

    let Some((part3, ext)) = parts[3].split_once('.') else {
      warn!("[TRACE] Invalid file name format: {}", parts[3]);
      warn!("[TRACE] Expected format: <part3>.<ext>");
      return Err(AppError::ChatFileError(format!(
        "Invalid file name {}",
        parts[3]
      )));
    };
    info!("[TRACE] Split filename: part3={}, ext={}", part3, ext);

    let hash = format!("{}{}{}", parts[1], parts[2], part3);
    info!("[TRACE] Constructed hash: {}", hash);
    info!(
      "[TRACE] Final ChatFile: workspace_id={}, hash={}, ext={}",
      ws_id, hash, ext
    );

    Ok(Self {
      workspace_id: ws_id,
      hash,
      ext: ext.to_string(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn chat_file_new_should_work() {
    let file = ChatFile::new(1, "test.txt", b"hello");
    assert_eq!(file.workspace_id, 1);
    assert_eq!(file.ext, "txt");
    assert_eq!(file.hash.len(), 40);
    assert_eq!(
      file.url(),
      "/files/1/aaf/4c6/1ddcc5e8a2dabede0f3b482cd9aea9434d.txt"
    );
    assert_eq!(
      file.hash_to_path(),
      "1/aaf/4c6/1ddcc5e8a2dabede0f3b482cd9aea9434d.txt"
    );
  }

  #[test]
  fn url_format_should_be_consistent() {
    let file = ChatFile::new(1, "test.txt", b"hello");
    let url = file.url();

    // verify URL format is correct
    assert!(url.starts_with("/files/1/"));
    assert!(url.ends_with(".txt"));

    // verify parsing consistency
    let parsed = ChatFile::from_str(&url).unwrap();
    assert_eq!(parsed.workspace_id, 1);
    assert_eq!(parsed.ext, "txt");
  }
}
