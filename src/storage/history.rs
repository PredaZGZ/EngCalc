use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
    pub is_error: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
    pub max_entries: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000,
        }
    }

    pub fn add(&mut self, expression: String, result: String, is_error: bool) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.entries.push(HistoryEntry {
            expression,
            result,
            is_error,
            timestamp,
        });

        if self.entries.len() > self.max_entries {
            let excess = self.entries.len() - self.max_entries;
            self.entries.drain(..excess);
        }
    }

    pub fn get_expressions(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.expression.as_str()).collect()
    }

    pub fn last_n(&self, n: usize) -> Vec<&HistoryEntry> {
        let len = self.entries.len();
        if len <= n {
            self.entries.iter().collect()
        } else {
            self.entries[len - n..].iter().collect()
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::history_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::history_path()?;
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        let history: History = serde_json::from_str(&content)?;
        Ok(history)
    }

    fn history_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("engcalc");
        Ok(dir.join("history.json"))
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
