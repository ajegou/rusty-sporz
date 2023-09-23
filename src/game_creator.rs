use std::collections::BTreeMap;
use std::collections::HashMap;
use std::cmp;
use std::fmt;
use std::error;

use rand::thread_rng;
use rand::seq::SliceRandom;

use crate::game::GameStatus;
use crate::interface::Interface;
use crate::player::Player;
use crate::role::Role;

struct GameCreator<'a> {
  debug: bool,
  interface: &'a mut Interface,
  id_keys: Vec<String>,
  player_names: BTreeMap<String, String>, // want a sorted map for simpler debug
  custom_roles: Option<HashMap<Role, usize>>,
  ship_name: Option<String>,
}

impl <'a> GameCreator<'a> {
  pub fn new (interface: &'a mut Interface, debug: bool) -> GameCreator<'a> {
    GameCreator {
      debug,
      interface,
      id_keys: generaye_keys(debug),
      player_names: BTreeMap::new(),
      custom_roles: None,
      ship_name: None,
    }
  }

  pub fn name_ship (&mut self) {
    let name = self.interface.user_non_empty_input("Quel est le nom de votre vaisseau?");
    self.ship_name = Some(name);
  }

  pub fn add_player (&mut self) {
    let name = loop {
      let name = self.interface.user_non_empty_input("Sous quel dénominatif souhaitez-vous être identifié·e?");
      if self.player_names.contains_key(&name) {
        println!("Désolé, ce dénominatif n'est pas disponible");
        continue;
      }
      break name;
    };
    let key = self.id_keys.pop().unwrap().to_string();
    self.interface.user_validate(format!("{name}, votre code secret est: '{key}', ne l'oubliez pas! Vous en aurez besoin pour vous identifier.").as_str());
    self.player_names.insert(name, key);
  }

  pub fn remove_player (&mut self) {
    if self.player_names.len() == 0 {
      self.interface.user_validate("Désolé, il n'y a aucun membre d'équipage à supprimer");
      self.interface.clear_terminal();
    } else {
      let selected = self.interface.user_select_from(self.player_names.keys());
      let key = self.player_names.remove(&selected.clone());
      self.id_keys.push(key.unwrap()); // The key cannot not be there
    }
  }

  pub fn update_roles (&mut self) {
    let all_roles = vec![Role::Patient0, Role::Psychologist, Role::Physician, Role::Geneticist, Role::ITEngineer, Role::Spy, Role::Hacker, Role::Traitor, Role::Astronaut];
    match &mut self.custom_roles {
      None => {
        println!("La partie est configurée pour utiliser les roles par défaut:");
      },
      Some(_) => {
        println!("La partie est configurée pour utiliser des roles personalisés:");
      },
    }

    let default_roles = self.get_default_roles();
    let roles = self.custom_roles.as_ref().unwrap_or(&default_roles);
    for role in &all_roles {
      let count = roles.get(role).unwrap_or(&0);
      if role == &Role::Patient0 && count < &1 {
        println!("* {}: {} -- Attention, jouer sans {} risque de mener à une partie très courte", role, count, role);
      } else if role == &Role::Physician && count < &2 {
        println!("* {}: {} -- Attention, jouer avec moins de 2 {} est très difficile", role, count, role);
      } else if role == &Role::Hacker && count >= &1 {
        let hackable_roles = roles.get(&Role::Spy).unwrap_or(&0)
          + roles.get(&Role::Psychologist).unwrap_or(&0)
          + roles.get(&Role::Geneticist).unwrap_or(&0);
        if hackable_roles < 2 {
          println!("* {}: {} -- Attention, peu de cibles disponibles pour le {}: {}", role, count, role, hackable_roles);
        }
      } else {
        println!("* {}: {}", role, count);
      }
    }
    println!();
    
    let modify = "Modifier les roles à utiliser";
    let use_default = "Utiliser les roles par défaut";
    let ret = "Retour";
    let choices = vec![modify, use_default, ret];
    let choice = self.interface.user_select_from(choices.iter());
    match *choice {
      ref x if x == &ret => (),
      ref x if x == &use_default => self.custom_roles = None,
      ref x if x == &modify => {
        if self.custom_roles.is_none() {
          self.custom_roles = Some(default_roles);
        }
        println!("");
        println!("Quel role voulez vous modifier?");
        let role = self.interface.user_select_from(all_roles.iter());
        let count = loop {
          let count = self.interface.user_non_empty_input(format!("Combient de {role} voulez vous?").as_str());
          let count = count.parse::<usize>();
          if let Ok(count) = count {
            break count;
          }
          println!("Avec un nombre ce serait pas mal!")
        };
        self.custom_roles.as_mut().unwrap().insert(*role, count);
        self.update_roles();
      }
      _ => panic!(), // beurk
    }
  }

  fn get_roles (&self) -> Vec<Role> {
    let default_roles = self.get_default_roles(); // lame, but not sure how to do otherwise
    let roles_map = self.custom_roles.as_ref().unwrap_or(&default_roles);

    let mut roles = Vec::new();
    for (role, count) in roles_map.iter() {
      for _ in 0..*count {
        roles.push(role.clone());
      }
    }
    if self.debug { // Keep the roles ordered when debugging
      roles.sort();
    } else {
      roles.shuffle(&mut thread_rng());
    }
    return roles;
  }

  pub fn get_default_roles (&self) -> HashMap<Role, usize> {
    let mut roles = HashMap::new();
    roles.insert(Role::Patient0, 1);
    roles.insert(Role::Physician, 2);
    roles.insert(Role::Psychologist, 1);
    roles.insert(Role::ITEngineer, 1);
    roles.insert(Role::Spy, 1);
    // Add astronauts for a 7 players game if there are less registered
    let astronauts = cmp::max(7, self.player_names.len()) - roles.values().sum::<usize>();
    roles.insert(Role::Astronaut, astronauts);
    return roles;
  }

  pub fn can_create_game (&mut self) -> bool {
    if self.ship_name == None {
      self.interface.user_validate("Vous devez donner un nom à votre vaisseau");
      return false;
    }
    if self.player_names.len() < 7 {
      self.interface.user_validate("Désolé, il vous faut au moins 7 joueurs pour jouer");
      return false;
    }
    let roles = self.get_roles();
    if self.player_names.len() != roles.len() {
      self.interface.user_validate(format!("Le nombre de roles ({}) doit correspondre au nombre de joueurs ({})", roles.len(), self.player_names.len()).as_str());
      return false;
    }
    return true;
  }

  pub fn create_game (self) -> Result<GameStatus, Box<dyn error::Error>> {
    let mut roles = self.get_roles();
    let has_geneticist = roles.contains(&Role::Geneticist);

    let mut next_user_id = 0;
    let mut players: Vec<Player> = Vec::new();
    for (name, key) in self.player_names {
      let role = roles.pop().unwrap();
      let player = Player::new(next_user_id, key, name, role);
      players.push(player);
      next_user_id += 1;
    }

    if has_geneticist {
      let mut potential_host_and_resilient = players.iter_mut()
        .filter(|player| player.role != Role::Patient0 && player.role != Role::Physician)
        .collect::<Vec<&mut Player>>();
      if !self.debug { // No random when debugging
        potential_host_and_resilient.shuffle(&mut thread_rng());
      }
      potential_host_and_resilient.pop().unwrap().host = true;
      potential_host_and_resilient.pop().unwrap().resilient = true;
    }

    Ok(GameStatus::new(self.ship_name.unwrap(), players, self.debug))
  }
}

pub fn create_game (interface: &mut Interface, debug: bool) -> Result<GameStatus, Box<dyn error::Error>> {
  let mut game_creator = GameCreator::new(interface, debug);

  enum Options {
    NameShip,
    AddPlayer,
    RemovePlayer,
    UpdateRoles,
    StartGame,
  }
  impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
        Options::NameShip => write!(f, "Nommer votre vaisseau")?,
        Options::AddPlayer => write!(f, "Ajouter un membre d'équipage")?,
        Options::RemovePlayer => write!(f, "Supprimer un membre d'équipage")?,
        Options::UpdateRoles => write!(f, "Selectionner la liste des roles")?,
        Options::StartGame => write!(f, "Commencer la partie")?,
      };
      return Ok(());
    }
  }

  loop {
    game_creator.interface.clear_terminal();
    let names = game_creator.player_names.keys().map(|name| name.clone()).collect::<Vec<String>>().join(", ");
    println!("Liste des membres d'équipage actifs: [{names}]");
    println!("Que souhaitez vous faire?");

    let options_list = vec![Options::NameShip, Options::AddPlayer, Options::RemovePlayer, Options::UpdateRoles, Options::StartGame];
    match game_creator.interface.user_select_from(options_list.iter()) {
      Options::NameShip => game_creator.name_ship(),
      Options::AddPlayer => game_creator.add_player(),
      Options::RemovePlayer => game_creator.remove_player(),
      Options::UpdateRoles => game_creator.update_roles(),
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
