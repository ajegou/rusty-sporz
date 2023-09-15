use crate::player::{Player, PlayerId};
use crate::action::ActionType;

pub struct GameStatus {
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
    }
  }

  pub fn get_alive_players(&self) -> Vec<&Player> {
    return self.players
      .iter()
      .filter(|player| player.alive)
      .collect();
  }

  pub fn get_current_player<'a>(&'a self) -> &'a Player { // panics if there is no current player
    &self.players[self.current_player_id.unwrap()]
  }

  pub fn get_mut_current_player<'a>(&'a mut self) -> &'a mut Player { // panics if there is no current player
    &mut self.players[self.current_player_id.unwrap()]
  }

  pub fn get_current_target<'a>(&'a self, action: &ActionType) -> Option<&PlayerId> {
    self.get_current_player().get_target(action)
  }

  pub fn get_current_target_p(&self, action: &ActionType) -> Option<&Player> {
    self.get_current_player().get_target(action).map(|player_id| &self.players[*player_id])
  }

  pub fn set_current_target(&mut self, action: &ActionType, target: Option<PlayerId>) {
    self.get_mut_current_player().set_target(action, target);
  }


  pub fn set_current_target_p(&mut self, action: &ActionType, target: &Option<&Player>) {
    self.get_mut_current_player().set_target(action, target.map(|player| player.id));
  }

  pub fn prepare_new_turn(&mut self) {
    self.players.iter_mut().for_each(|player| player.prepare_new_turn());
  }
}