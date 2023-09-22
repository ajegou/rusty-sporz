use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  pub date: u32,
  pub source: String,
  pub content: String,
}

impl Message {
  pub fn to_string(&self) -> String {
    return format!("* Day {} from [{}]: {}", self.date, self.source, self.content);
  }
}
