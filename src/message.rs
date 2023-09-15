
#[derive(Debug)]
pub struct Message {
  date: u32,
  source: String,
  content: String,
}

impl Message {
  pub fn to_string(&self) -> String {
    return format!("* Day {} from [{}]: {}", self.date, self.source, self.content);
  }
}