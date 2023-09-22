use std::slice::Iter;
use std::error;
use std::time::SystemTime;

use serde::{Serialize, Deserialize};

use crate::backup::{backup_game, restore_game};
use crate::message::Message;
use crate::player::{Player, PlayerId};
use crate::action::ActionType;

#[derive(Serialize, Deserialize)]
pub struct GameStatus {
  #[serde(skip_deserializing)]
  creation: u64, // this is to avoid collisions after loading a backup, so we want it unique
  name: String,
  date: u32,
  players: Vec<Player>,
  current_player_id: Option<PlayerId>,
  debug: bool,
}

impl GameStatus {
  pub fn new (name: String, players: Vec<Player>, debug: bool) -> GameStatus {
    GameStatus{
      creation: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
      name,
      players,
      current_player_id: None,
      debug,
      date: 1,
    }
  }

  pub fn restore_from_backup (path: &String) -> Result<GameStatus, Box<dyn error::Error>> {
    let mut game = restore_game(path)?;
    game.creation = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    return Ok(game);
  }
}

pub struct PlayerTurn<'a> {
  game: &'a mut GameStatus,
  current_player_id: PlayerId,
}

pub trait Game {
  fn debug(&self) -> bool;
  fn backup(&self, path: &str) -> Result<(), Box<dyn error::Error>>;
  fn get_name(&self) -> &str;
  fn get_date(&self) -> u32;
  fn ended(&self) -> bool;
  fn prepare_new_turn(&mut self);

  fn get_player_id_from_key(&self, key: String) -> Option<PlayerId>;

  fn get_player(&self, id: PlayerId) -> &Player;
  fn get_mut_player(&mut self, id: PlayerId) -> &mut Player;

  fn get_players(&self) -> Vec<&Player>; // returns only alive players
  fn get_mut_players(&mut self) -> Vec<&mut Player>; // returns only alive players
  fn get_all_players(&self) -> Iter<'_, Player>;
  fn get_player_ids(&self, predicate: &dyn Fn(&&&Player) -> bool) -> Vec<PlayerId>; // returns only alive players

  fn broadcast (&mut self, message: Message);
  fn limited_broadcast(&mut self, message: Message, predicate: &dyn Fn(&&mut &mut Player) -> bool);

  fn get_current_player_id(&self) -> Option<PlayerId>;
  fn set_current_player_id(&mut self, player: Option<PlayerId>);
  fn get_player_game<'a> (&'a mut self, current_player_id: PlayerId) -> PlayerTurn<'a>;
}

pub trait PlayerGame: Game {
  fn get_current_player<'a>(&'a self) -> &'a Player;
  fn get_mut_current_player<'a>(&'a mut self) -> &'a mut Player;
  fn get_current_target(&self, action: &ActionType) -> Option<&Player>;
  fn set_current_target(&mut self, action: &ActionType, target: Option<PlayerId>);
  fn set_current_target_p(&mut self, action: &ActionType, target: &Option<&Player>);
}

impl Game for GameStatus {
  fn debug(&self) -> bool {
    return self.debug;
  }

  fn backup(&self, path: &str) -> Result<(), Box<dyn error::Error>> {
    return backup_game(self, format!("{path}sporz-{}-{}-day-{}", &self.name, &self.creation, &self.date));
  }

  fn get_name(&self) -> &str {
    return self.name.as_str();
  }

  fn get_date(&self) -> u32 {
    return self.date;
  }

  fn get_player_id_from_key(&self, key: String) -> Option<PlayerId> {
    match self.players.iter().find(|player| player.key == key) {
      Some(player) => Some(player.id.clone()),
      None => None,
    }
  }

  fn get_player(&self, id: PlayerId) -> &Player {
    return id.get_player(&self.players);
  }

  fn get_mut_player(&mut self, id: PlayerId) -> &mut Player {
    return id.get_mut_player(&mut self.players);
  }

  fn get_all_players(&self) -> Iter<'_, Player>{
    return self.players.iter();
  }

  fn get_players(&self) -> Vec<&Player> {
    return self.players
      .iter()
      .filter(|player| player.alive)
      .collect();
  }

  fn get_mut_players(&mut self) -> Vec<&mut Player> {
    return self.players
      .iter_mut()
      .filter(|player| player.alive)
      .collect();
  }

  fn broadcast (&mut self, message: Message) {
    for player in &mut self.players {
      player.messages.push(message.clone()); // Maybe use borrowing instead of clone, but needs lifetime
    }
  }

  fn limited_broadcast(&mut self, message: Message, predicate: &dyn Fn(&&mut &mut Player) -> bool) {
    for player in self.get_mut_players().iter_mut().filter(predicate) {
      player.send_message(message.clone());
    }
  }

  fn get_player_ids(&self, predicate: &dyn Fn(&&&Player) -> bool) -> Vec<PlayerId> {
    return self.get_players().iter().filter(predicate).map(|player| player.id).collect();
  }

  fn prepare_new_turn(&mut self) {
    self.players.iter_mut().for_each(|player| player.prepare_new_turn());
    self.date += 1;
  }

  fn ended(&self) -> bool {
    let healthy_players = self.get_players().iter().filter(|player| !player.infected).count();
    healthy_players == 0 || healthy_players == self.get_players().len()
  }

  fn get_current_player_id(&self) -> Option<PlayerId> {
    self.current_player_id
  }

  fn set_current_player_id(&mut self, player: Option<PlayerId>) {
    self.current_player_id = player;
  }

  fn get_player_game<'a> (&'a mut self, current_player_id: PlayerId) -> PlayerTurn<'a> {
    return PlayerTurn{ game: self, current_player_id };
  }
}

impl <'b> PlayerGame for PlayerTurn<'b> {
  fn get_current_player<'a>(&'a self) -> &'a Player {
    &self.current_player_id.get_player(&self.game.players)
  }

  fn get_mut_current_player<'a>(&'a mut self) -> &'a mut Player {
    self.current_player_id.get_mut_player(&mut self.game.players)
  }

  fn get_current_target(&self, action: &ActionType) -> Option<&Player> {
    self.get_current_player().get_target(action).map(|player_id| player_id.get_player(&self.game.players))
  }

  fn set_current_target(&mut self, action: &ActionType, target: Option<PlayerId>) {
    self.get_mut_current_player().set_target(action, target);
  }

  fn set_current_target_p(&mut self, action: &ActionType, target: &Option<&Player>) {
    // This one is problematic, because it needs a mutable game, but the &player is borrowed from the game...
    self.get_mut_current_player().set_target(action, target.map(|player| player.id));
  }
}

impl <'b> Game for PlayerTurn<'b> { // Proxy everything to self.game
  fn debug(&self) -> bool {
    self.game.debug()
  }

  fn backup(&self, path: &str) -> Result<(), Box<dyn error::Error>> {
    self.game.backup(path)
  }

  fn get_name(&self) -> &str {
    self.game.get_name()
  }

  fn get_date(&self) -> u32 {
    self.game.get_date()
  }

  fn get_player_id_from_key(&self, key: String) -> Option<PlayerId> {
    self.game.get_player_id_from_key(key)
  }

  fn get_player(&self, id: PlayerId) -> &Player {
    self.game.get_player(id)
  }

  fn get_mut_player(&mut self, id: PlayerId) -> &mut Player {
    self.game.get_mut_player(id)
  }

  fn get_all_players(&self) -> Iter<'_, Player> {
    self.game.get_all_players()
  }

  fn get_players(&self) -> Vec<&Player> {
    self.game.get_players()
  }

  fn get_mut_players(&mut self) -> Vec<&mut Player> {
    self.game.get_mut_players()
  }

  fn broadcast (&mut self, message: Message) {
    self.game.broadcast(message)
  }

  fn limited_broadcast(&mut self, message: Message, predicate: &dyn Fn(&&mut &mut Player) -> bool) {
    self.game.limited_broadcast(message, predicate)
  }

  fn get_player_ids(&self, predicate: &dyn Fn(&&&Player) -> bool) -> Vec<PlayerId> {
    self.game.get_player_ids(predicate)
  }

  fn prepare_new_turn(&mut self) {
    self.game.prepare_new_turn()
  }

  fn ended(&self) -> bool {
    self.game.ended()
  }

  fn get_current_player_id(&self) -> Option<PlayerId> {
    self.game.get_current_player_id()
  }

  fn set_current_player_id(&mut self, player: Option<PlayerId>) {
    self.game.set_current_player_id(player)
  }

  fn get_player_game<'a> (&'a mut self, current_player_id: PlayerId) -> PlayerTurn<'a> {
    self.game.get_player_game(current_player_id)
  }
}
