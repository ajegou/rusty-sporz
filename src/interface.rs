use std::io;
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

pub fn user_ask_and_validate(message: &str) -> Result<Option<String>, io::Error> {
  let mut input = String::new();

  while input.len() == 0 {
      print!("{message} ");
      io::stdout().flush()?;
      io::stdin().read_line(&mut input)?;
      input = input.trim().to_string();
  }

  print!("Your entered '{input}', type 'y' to validate: ");
  io::stdout().flush()?;

  let mut validation = String::new();
  io::stdin().read_line(&mut validation)?;
  validation = validation.trim().to_string();

  if validation == "y" {
      return Ok(Some(input));
  }
  return Ok(None);
}

pub fn clear_terminal() {
  if unsafe { DEBUG } { // do better
      print!("\n\n\n");
  } else {
      print!("{}[2J", 27 as char);
  }
}