use std::{io, collections::HashMap};
use std::io::Write;

use crate::{player::Player, action::Action, action::Action::{UserAction, GeneralAction}, DEBUG};

pub mod colors;

pub fn user_select_target<'a>(targets_list: &'a Vec<&'a Player>) -> Option<&'a Player> {
  for (idx, target) in targets_list.iter().enumerate() {
      println!("{idx}) {}", target.name);
  }
  println!("{}) {}", targets_list.len(), "Aucun");
  let accepted_answers: Vec<String> = (0..targets_list.len() + 1)
      .map(|value| { value.to_string() })
      .collect();
  let choice: usize = user_choice("Quel est votre choix?", accepted_answers).parse().unwrap();
  if choice == targets_list.len() {
      return None;
  }
  return Some(targets_list[choice]);
}

pub fn user_select_action<'a>(actions_list: &'a Vec<Action>) -> &'a Action {
  for (idx, action) in actions_list.iter().enumerate() {
      match action { // Hmmm... weird...
          UserAction(description, _) => println!("{idx}) {}", description),
          GeneralAction(description, _) => println!("{idx}) {}", description),
      }
  }
  let accepted_answers: Vec<String> = (0..actions_list.len())
      .map(|value| { value.to_string() })
      .collect();
  let choice: usize = user_choice("Quel est votre choix?", accepted_answers).parse().unwrap();
  return &actions_list[choice];
}

pub fn user_select<'a, T: std::fmt::Display> (options_list: impl Iterator<Item = &'a T>) -> &'a T {
  let mut options_by_idx: HashMap<String, &T> = HashMap::new();
  let mut idx = 1;
  for option in options_list {
      println!("{idx}) {}", option);
      options_by_idx.insert(idx.to_string(), option);
      idx += 1;
  }

  println!();
  loop {
      let mut input = String::new();
      print!("Quel est votre choix? ");
      io::stdout().flush().unwrap();
      io::stdin().read_line(&mut input).unwrap();
      input = input.trim().to_string();
      if let Some(selection) = options_by_idx.remove(&input) {
        return selection;
      }
  }
}

fn user_choice(message: &str, accepted_answers: Vec<String>) -> String {
  println!();
  loop {
      let mut input = String::new();
      print!("{message} ");
      io::stdout().flush().unwrap();
      io::stdin().read_line(&mut input).unwrap();
      input = input.trim().to_string();
      if accepted_answers.contains(&input) {
          return input;
      }
  }
}

pub fn user_validate(message: &str) {
  print!("{message} ");
  io::stdout().flush().unwrap();
  io::stdin().read_line(&mut String::new()).unwrap();
}

pub fn user_non_empty_input(message: &str) -> String {
  loop {
      let mut input = String::new();
      print!("{message} ");
      io::stdout().flush().unwrap();
      io::stdin().read_line(&mut input).unwrap();
      input = input.trim().to_string();
      if input.len() > 0 {
          return input;
      }
  }
}

pub fn clear_terminal() {
  if unsafe { DEBUG } { // do better
      print!("\n\n\n");
  } else {
      print!("{}[2J", 27 as char);
  }
}