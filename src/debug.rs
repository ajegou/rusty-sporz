use crate::{interface::Interface, game::Game};


pub static DEBUG_PLAYER_NAMES: [&str; 9] = [
  "Mal",
  "Zoe",
  "Wash",
  "Inara",
  "Kaylee",
  "Jayne",
  "Book",
  "Simon",
  "River",
];

pub fn mock_game_creator (interface: &mut Interface) {
  let mut inputs = Vec::new();
  inputs.push(String::from("1\n"));
  inputs.push(String::from("Koursk\n"));
  for name in DEBUG_PLAYER_NAMES {
    inputs.push(String::from("2\n"));
    inputs.push(format!("{name}\n"));
    inputs.push(String::from(""));
  }
  inputs.push(String::from("4\n"));
  interface.mock(inputs);
}

pub fn mock_game_vote_tie (interface: &mut Interface, game: &mut dyn Game) { // create votes to have a tie
  // This only works if the options (here the names of the players to vote for) are always proposed in the same order
  let mut inputs = Vec::new();
  let get_player_key = |name: &str| &game.get_players().iter().find(|player| player.name == name).unwrap().key;
  let mut idx = 0;
  for name in DEBUG_PLAYER_NAMES {
    let player_key = get_player_key(name);
    inputs.push(String::from("0\n")); // log-in
    inputs.push(String::from(player_key)); // key
    inputs.push(String::from("1\n")); // vote to eliminate
    inputs.push((idx / 3).to_string());
    inputs.push(String::from("0\n")); // exit
    idx += 1;
  }
  interface.mock(inputs);
}