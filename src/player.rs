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
  pub actions: HashMap<ActionType, PlayerId>,

  // status
  pub alive: bool,
  pub infected: bool,
  pub paralyzed: bool,
  pub death_cause: Option<String>,
  pub has_connected_today: bool,
}

impl Player {
  pub fn new(id: usize, key: String, name: String, role: Role) -> Player {
    let infected = role == Role::Patient0;
    Player {
      id: PlayerId { id: id },
      key,
      name,
      role,
      alive: true,
      infected,
      paralyzed: false,
      death_cause: None,
      messages: Vec::new(),
      actions: HashMap::new(),
      has_connected_today: false,
    }
  }

  pub fn prepare_new_turn(&mut self) {
    self.actions = HashMap::new();
    self.has_connected_today = false;
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
