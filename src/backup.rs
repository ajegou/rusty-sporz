use std::fs;
use std::fs::File;
use std::io;
use std::error;

use crate::game::GameStatus;

pub fn backup_game (game: &GameStatus, path: String) -> Result<(), Box<dyn error::Error>> {
  let serialized = serde_json::to_string(game)?;
  return Ok(fs::write(path, serialized)?);
}

pub fn restore_game (path: &String) -> Result<GameStatus, Box<dyn error::Error>> {
  let input = File::open(path)?;
  let reader = io::BufReader::new(input);
  return Ok(serde_json::from_reader(reader)?);
}