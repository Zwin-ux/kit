//! A session file: one JSON line per event, readable by people and `jq`.
//! The file is the agent's only memory; nothing is kept elsewhere.

use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Appends events to `<dir>/<id>.jsonl`.
#[derive(Debug)]
pub struct SessionLog {
    path: PathBuf,
    file: std::fs::File,
}

impl SessionLog {
    /// Create the folder and the file.
    pub fn create(dir: &Path, id: &str) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{id}.jsonl"));
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        Ok(Self { path, file })
    }

    /// Where the file is.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one record with a timestamp (ms since the epoch).
    pub fn write(&mut self, record: &impl Serialize) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let mut v = serde_json::to_value(record).unwrap_or_default();
        if let Some(obj) = v.as_object_mut() {
            obj.insert("ts".into(), ts.into());
        }
        // A failed write must not stop the agent; the event still reached the screen.
        let _ = writeln!(self.file, "{v}");
    }
}
