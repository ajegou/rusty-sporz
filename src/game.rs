use crate::message::Message;
use crate::player::{Player, PlayerId};
use crate::action::ActionType;

pub struct GameStatus {
  date: u32,
  pub players: Vec<Player>,
  pub current_player_id: Option<PlayerId>,
  pub ended: bool,
  pub day: bool,
  pub debug: bool,
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

  pub fn get_date(&self) -> u32 {
    return self.date;
  }

  pub fn get_alive_players(&self) -> Vec<&Player> {
    return self.players
      .iter()
      .filter(|player| player.alive)
      .collect();
  }

  pub fn get_mut_alive_players(&mut self) -> Vec<&mut Player> {
    return self.players
      .iter_mut()
      .filter(|player| player.alive)
      .collect();
  }

  pub fn get_current_player<'a>(&'a self) -> &'a Player { // panics if there is no current player
    &self.players[self.current_player_id.unwrap()]
  }

  pub fn get_mut_current_player<'a>(&'a mut self) -> &'a mut Player { // panics if there is no current player
    &mut self.players[self.current_player_id.unwrap()]
  }

  pub fn get_current_target(&self, action: &ActionType) -> Option<&Player> {
    self.get_current_player().get_target(action).map(|player_id| &self.players[*player_id])
  }

  pub fn set_current_target(&mut self, action: &ActionType, target: Option<PlayerId>) {
    self.get_mut_current_player().set_target(action, target);
  }

  pub fn set_current_target_p(&mut self, action: &ActionType, target: &Option<&Player>) {
    // This one is problematic, because it needs a mutable game, but the &player is borrowed from the game...
    self.get_mut_current_player().set_target(action, target.map(|player| player.id));
  }

  pub fn prepare_new_turn(&mut self) {
    self.players.iter_mut().for_each(|player| player.prepare_new_turn());
    self.date += 1;
  }

  pub fn broadcast (&mut self, message: Message) {
    for player in &mut self.players {
      player.messages.push(message.clone()); // Maybe use borrowing instead of clone, but needs lifetime
    }
  }

  pub fn limited_broadcast<P>(&mut self, message: Message, predicate: P)
  where
    P: FnMut(&&mut &mut Player) -> bool,
  {
    for player in self.get_mut_alive_players().iter_mut().filter(predicate) {
      player.send_message(message.clone());
    }
  }

  pub fn get_player_ids<P>(&self, predicate: P) -> Vec<PlayerId>
  where
    P: FnMut(&&&Player) -> bool,
  {
    return self.get_alive_players().iter().filter(predicate).map(|player| player.id).collect();
  }
}