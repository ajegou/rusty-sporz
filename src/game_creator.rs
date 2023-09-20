use std::collections::HashMap;
use std::fmt;
use std::error;

use rand::thread_rng;
use rand::seq::SliceRandom;

use crate::game::GameStatus;
use crate::interface::clear_terminal;
use crate::interface::user_non_empty_input;
use crate::interface::user_select;
use crate::interface::user_validate;
use crate::player::Player;
use crate::role::Role;

struct GameCreator {
  debug: bool,
  id_keys: Vec<String>,
  player_names: HashMap<String, String>,
  custom_roles: Option<Vec<Role>>,
}

impl GameCreator {
  pub fn new (debug: bool) -> GameCreator {
    GameCreator {
      debug,
      id_keys: generaye_keys(debug),
      player_names: HashMap::new(),
      custom_roles: None,
    }
  }
  
  pub fn add_player (&mut self) {
    let name = loop {
      let name = user_non_empty_input("Sous quel dénominatif souhaitez-vous être identifié·e?");
      if self.player_names.contains_key(&name) {
        println!("Désolé, ce dénominatif n'est pas disponible");
        continue;
      }
      break name;
    };
    let key = self.id_keys.pop().unwrap().to_string();
    println!("{name}, votre code secret est: '{key}', ne l'oubliez pas! Vous en aurez besoin pour vous identifier.");
    if !self.debug {
        user_validate("");
    }
    self.player_names.insert(name, key);
  }

  pub fn remove_player (&mut self) {
    if self.player_names.len() == 0 {
      user_validate("Désolé, il n'y a aucun membre d'équipage à supprimer");
      clear_terminal();
    } else {
      let selected = user_select(self.player_names.keys());
      let key = self.player_names.remove(&selected.clone());
      self.id_keys.push(key.unwrap()); // The key cannot not be there
    }
  }

  pub fn get_default_roles (&mut self) -> Vec<Role> {
    let mut roles = Vec::new();
    roles.push(Role::Patient0);
    roles.push(Role::Physician);
    roles.push(Role::Physician);
    roles.push(Role::Psychologist);
    roles.push(Role::ITEngineer);
    roles.push(Role::Spy);
    roles.push(Role::Astronaut);
    while roles.len() < self.player_names.len() {
      roles.push(Role::Astronaut);
    }
    roles.shuffle(&mut thread_rng());
    return roles;
  }

  pub fn can_create_game (&mut self) -> bool {
    if self.player_names.len() < 7 {
      user_validate("Désolé, il vous faut au moins 7 joueurs pour jouer");
      return false;
    }
    if let Some(roles) = &self.custom_roles {
      if self.player_names.len() != roles.len() {
        user_validate(format!("Le nombre de roles ({}) doit correspondre au nombre de joueurs ({})", roles.len(), self.player_names.len()).as_str());
        return false;
      }
    }
    return true;
  }

  pub fn create_game (mut self) -> Result<GameStatus, Box<dyn error::Error>> {
    let mut roles = match self.custom_roles {
      Some(roles) => roles,
      None => self.get_default_roles(),
    };

    let mut next_user_id = 0;
    let mut players: Vec<Player> = Vec::new();
    for (name, key) in self.player_names {
        let role = roles.pop().unwrap();
        let player = Player::new(next_user_id, key, name, role);
        players.push(player);
        next_user_id += 1;
    }
    Ok(GameStatus::new(players, self.debug))
  }
}

pub fn create_game (args: Vec<String>) -> Result<GameStatus, Box<dyn error::Error>> {
  let debug = args.contains(&String::from("--debug"));
  let mut game_creator = GameCreator::new(debug);

  enum Options {
    AddPlayer,
    RemovePlayer,
    StartGame,
  }
  impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
        Options::AddPlayer => write!(f, "Ajouter un membre d'équipage")?,
        Options::RemovePlayer => write!(f, "Supprimer un membre d'équipage")?,
        Options::StartGame => write!(f, "Commencer la partie")?,
      };
      return Ok(());
    }
  }

  loop {
    clear_terminal();
    let names = game_creator.player_names.keys().map(|name| name.clone()).collect::<Vec<String>>().join(", ");
    println!("Liste des membres d'équipage actifs: [{names}]");
    println!("Que souhaitez vous faire?");
    
    match user_select(vec![Options::AddPlayer, Options::RemovePlayer, Options::StartGame].iter()) {
      Options::AddPlayer => game_creator.add_player(),
      Options::RemovePlayer => game_creator.remove_player(),
      Options::StartGame => {
        if game_creator.can_create_game() {
          return game_creator.create_game();
        }
      },
    }
  }
  
}


fn generaye_keys (debug: bool) -> Vec<String> {
  let mut keys: Vec<String> = (100..1000).map(|number| number.to_string()).collect();
  if debug {
      keys.reverse(); // Want ids to be 100 101 102 ... in debug
  } else {
      keys.shuffle(&mut thread_rng());
  }
  return keys;
}
