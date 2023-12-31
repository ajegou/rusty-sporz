use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::role::Role;
use crate::message::Message;
use crate::action::ActionType;

#[derive(Debug, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerId {
  id: usize,
}

impl PartialEq for PlayerId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PlayerId {
  pub fn get_player<'a>(&self, players: &'a Vec<Player>) -> &'a Player {
    return &players[self.id];
  }
  pub fn get_mut_player<'a>(&self, players: &'a mut Vec<Player>) -> &'a mut Player {
    return &mut players[self.id];
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
  pub id: PlayerId,
  pub key: String,
  pub name: String,
  pub role: Role,
  pub messages: Vec<Message>,
  pub host: bool,
  pub resilient: bool,

  // status
  pub alive: bool,
  pub infected: bool,
  pub paralyzed: bool,
  pub death_date: Option<u32>,
  pub death_cause: Option<String>,
  pub auto_cure_physician: bool,
  pub auto_kill_physician: bool,
  pub physician_kill: bool,
  pub mutant_kill: bool,

  // daily data
  pub has_connected_today: bool,
  pub actions: HashMap<ActionType, PlayerId>,
  pub spy_info: SpyData,
  pub hacker_target: Option<Role>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct SpyData {
  pub woke_up: bool,
  pub was_cured: bool, // only if it had an effect
  pub was_infected: bool, // only if it had an effect
  pub was_paralyzed: bool,
  pub was_psychoanalyzed: bool,
}

impl Player {
  pub fn new(id: usize, key: String, name: String, role: Role) -> Player {
    let infected = role == Role::Patient0;
    Player {
      id: PlayerId { id: id },
      key,
      name,
      role,
      host: false,
      resilient: false,
      alive: true,
      infected,
      paralyzed: false,
      death_date: None,
      death_cause: None,
      auto_cure_physician: true,
      auto_kill_physician: false,
      physician_kill: false,
      mutant_kill: false,
      messages: Vec::new(),
      has_connected_today: false,
      actions: HashMap::new(),
      spy_info: SpyData{ ..Default::default() },
      hacker_target: None,
    }
  }

  pub fn prepare_new_turn(&mut self) {
    self.actions = HashMap::new();
    self.has_connected_today = false;
    self.spy_info = SpyData{ ..Default::default() };
    self.hacker_target = None;
    self.physician_kill = false;
    self.mutant_kill = false;
  }

  pub fn die(&mut self, date: u32, death_cause: String) {
    self.alive = false;
    self.death_date = Some(date);
    self.death_cause = Some(death_cause);
  }

  pub fn get_target(&self, action: &ActionType) -> Option<&PlayerId> {
    return self.actions.get(action);
  }

  pub fn set_target(&mut self, action: &ActionType, target: Option<PlayerId>) {
    match target {
        Some(target) => self.actions.insert(*action, target),
        None => self.actions.remove(&action),
    };
  }

  pub fn get_death_cause(&self) -> &String {
    return self.death_cause.as_ref().unwrap();
  }

  pub fn send_message(&mut self, message: Message) {
    self.messages.push(message);
  }
}
