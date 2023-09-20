use std::slice::Iter;

use crate::message::Message;
use crate::player::{Player, PlayerId};
use crate::action::ActionType;

pub struct GameStatus {
  date: u32,
  players: Vec<Player>,
  pub current_player_id: Option<PlayerId>,
  pub ended: bool,
  pub day: bool,
  pub debug: bool,
}

pub struct PlayerTurn<'a> {
  pub game: &'a mut GameStatus,
  current_player_id: PlayerId,
}

pub trait Game {
  fn get_date(&self) -> u32;
  fn get_player_id_from_key(&self, key: String) -> Option<PlayerId>;
  fn get_player(&self, id: PlayerId) -> &Player;
  fn get_mut_player(&mut self, id: PlayerId) -> &mut Player;
  fn get_all_players(&self) -> Iter<'_, Player>;
  fn get_alive_players(&self) -> Vec<&Player>;
  fn get_mut_alive_players(&mut self) -> Vec<&mut Player>;
  fn broadcast (&mut self, message: Message);
  fn limited_broadcast(&mut self, message: Message, predicate: &dyn Fn(&&mut &mut Player) -> bool);
  fn get_player_ids(&self, predicate: &dyn Fn(&&&Player) -> bool) -> Vec<PlayerId>;
}

pub trait PlayerGame: Game {
  fn get_current_player<'a>(&'a self) -> &'a Player;
  fn get_mut_current_player<'a>(&'a mut self) -> &'a mut Player;
  fn get_current_target(&self, action: &ActionType) -> Option<&Player>;
  fn set_current_target(&mut self, action: &ActionType, target: Option<PlayerId>);
  fn set_current_target_p(&mut self, action: &ActionType, target: &Option<&Player>);
}

impl GameStatus {
  pub fn new (players: Vec<Player>) -> GameStatus {
    GameStatus{
      players,
      current_player_id: None,
      day: true,
      ended: false,
      debug: false,
      date: 1,
    }
  }

  pub fn prepare_new_turn(&mut self) {
    self.players.iter_mut().for_each(|player| player.prepare_new_turn());
    self.date += 1;
  }

  pub fn get_player_game<'a> (&'a mut self, current_player_id: PlayerId) -> PlayerTurn<'a> {
    return PlayerTurn{ game: self, current_player_id };
  }
}

impl Game for GameStatus {
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

  fn get_alive_players(&self) -> Vec<&Player> {
    return self.players
      .iter()
      .filter(|player| player.alive)
      .collect();
  }

  fn get_mut_alive_players(&mut self) -> Vec<&mut Player> {
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
    for player in self.get_mut_alive_players().iter_mut().filter(predicate) {
      player.send_message(message.clone());
    }
  }

  fn get_player_ids(&self, predicate: &dyn Fn(&&&Player) -> bool) -> Vec<PlayerId> {
    return self.get_alive_players().iter().filter(predicate).map(|player| player.id).collect();
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

  fn get_alive_players(&self) -> Vec<&Player> {
    self.game.get_alive_players()
  }

  fn get_mut_alive_players(&mut self) -> Vec<&mut Player> {
    self.game.get_mut_alive_players()
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
}
