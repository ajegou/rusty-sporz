use std::collections::HashMap;
use std::io::Error;
use std::error;

use rand::thread_rng;
use rand::seq::SliceRandom;

use crate::debug;
use crate::game::GameStatus;
use crate::interface::clear_terminal;
use crate::interface::user_ask_and_validate;
use crate::interface::user_validate;
use crate::player::Player;
use crate::role;
use crate::role::Role;

pub fn create_game (args: Vec<String>) -> Result<GameStatus, Box<dyn error::Error>> {
  let debug = args.contains(&String::from("--debug"));

  let players_names = get_players_list(debug)?;
  let number_of_players = u32::try_from(players_names.len()).unwrap();
  let mut roles = role::get_roles(number_of_players)?;
  roles.shuffle(&mut thread_rng());
  let players = create_players(players_names, roles);

  Ok(GameStatus::new(players, debug))
}

fn get_players_list(use_debug: bool) -> Result<HashMap<String, String>, Error> {
  let mut keys: Vec<usize> = (100..1000).collect();
  if use_debug {
      keys.reverse(); // Want ids to be 100 101 102 ... in debug
  } else {
      keys.shuffle(&mut thread_rng());
  }

  clear_terminal();
  let mut players: HashMap<String, String> = HashMap::new();
  loop {
      let message = "Enter your name (or enter DONE if no more players):";
      let input;
      if use_debug {
          if players.len() == debug::NAMES.len() {
              break;
          }
          input = Some(debug::NAMES[players.len()].to_string());
      } else {
          input = user_ask_and_validate(message)?;
      }
      match input {
          None => println!("Annulation"),
          Some(name) => {
              if name == "DONE" {
                  break;
              }
              if players.contains_key(&name) {
                  println!("Name '{name}' is already in use");
              } else {
                  let key = keys.pop().unwrap().to_string();
                  println!("Your secret key is {key}, do not forget it! You will need it to log-in later");
                  if !use_debug {
                      user_validate("");
                  }
                  players.insert(name, key);
                  clear_terminal();
              }
          }
      }
  }
  return Ok(players);
}

// Consumes both names_to_keys and roles
fn create_players<'a>(names_to_keys: HashMap<String, String>, mut roles: Vec<Role>) -> Vec<Player> {
  let mut next_user_id = 0;
  let mut players: Vec<Player> = Vec::new();
  for (name, key) in names_to_keys {
      let role = roles.pop().unwrap();
      let player = Player::new(next_user_id, key, name, role);
      players.push(player);
      next_user_id += 1;
  }
  return players;
}