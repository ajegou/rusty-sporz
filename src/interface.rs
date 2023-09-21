use std::{io, collections::HashMap};
use std::io::Write;

use crate::{player::Player, action::Action, action::Action::{UserAction, GeneralAction}};

pub mod colors;

pub struct Interface {
  debug: bool,
  input_mock: Vec<String>,
}

impl Interface {
  pub fn new (debug: bool) -> Interface {
    Interface {
      debug,
      input_mock: Vec::new(),
    }
  }

  pub fn mock (&mut self, mut inputs: Vec<String>) {
    if !self.input_mock.is_empty() {
      panic!("Trying to add mock while there is still some moco")
    }
    inputs.reverse();
    for input in inputs {
      self.input_mock.push(input);
    }
  }

  fn read_line (&mut self, input: &mut String) -> Result<usize, std::io::Error> {
    if let Some(next_mock) = self.input_mock.pop() {
      let bytes = next_mock.len();
      print!("{}", colors::Color::FgCyan.color(&next_mock));
      *input = next_mock;
      return Ok(bytes);
    } else {
      return io::stdin().read_line(input);
    }
  }

  pub fn user_select_target<'a>(&mut self, targets_list: &'a Vec<&'a Player>) -> Option<&'a Player> {
    for (idx, target) in targets_list.iter().enumerate() {
        println!("{idx}) {}", target.name);
    }
    println!("{}) {}", targets_list.len(), "Aucun");
    let accepted_answers: Vec<String> = (0..targets_list.len() + 1)
        .map(|value| { value.to_string() })
        .collect();
    let choice: usize = self.user_choice("Quel est votre choix?", accepted_answers).parse().unwrap();
    if choice == targets_list.len() {
        return None;
    }
    return Some(targets_list[choice]);
  }

  pub fn user_select_action<'a>(&mut self, actions_list: &'a Vec<Action>) -> &'a Action {
    for (idx, action) in actions_list.iter().enumerate() {
        match action { // Hmmm... weird...
            UserAction(description, _) => println!("{idx}) {}", description),
            GeneralAction(description, _) => println!("{idx}) {}", description),
        }
    }
    let accepted_answers: Vec<String> = (0..actions_list.len())
        .map(|value| { value.to_string() })
        .collect();
    let choice: usize = self.user_choice("Quel est votre choix?", accepted_answers).parse().unwrap();
    return &actions_list[choice];
  }

  pub fn user_select_from<'a, O: std::fmt::Display> (&mut self, options_list: impl Iterator<Item = &'a O>) -> &'a O {
    return self.user_select_from_with_custom_display(options_list, |x| *x);
  }

  pub fn user_select_from_with_custom_display<O, T: std::fmt::Display> (&mut self, options_list: impl Iterator<Item = O>, displayer: impl Fn(&O) -> T) -> O {
    let mut options_by_idx: HashMap<String, O> = HashMap::new();
    let mut idx = 1;
    for option in options_list {
        println!("{idx}) {}", displayer(&option));
        options_by_idx.insert(idx.to_string(), option);
        idx += 1;
    }

    println!();
    loop {
        let mut input = String::new();
        print!("Quel est votre choix? ");
        io::stdout().flush().unwrap();
        self.read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if let Some(selection) = options_by_idx.remove(&input) {
          return selection;
        }
    }
  }

  fn user_choice(&mut self, message: &str, accepted_answers: Vec<String>) -> String {
    println!();
    loop {
        let mut input = String::new();
        print!("{message} ");
        io::stdout().flush().unwrap();
        self.read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if accepted_answers.contains(&input) {
            return input;
        }
    }
  }

  pub fn user_validate(&mut self, message: &str) {
    print!("{message} ");
    io::stdout().flush().unwrap();
    self.read_line(&mut String::new()).unwrap();
  }

  pub fn user_non_empty_input(&mut self, message: &str) -> String {
    loop {
        let mut input = String::new();
        print!("{message} ");
        io::stdout().flush().unwrap();
        self.read_line(&mut input).unwrap();
        input = input.trim().to_string();
        if input.len() > 0 {
            return input;
        }
    }
  }

  pub fn clear_terminal(&self) {
    if self.debug {
        print!("\n##############################\n\n");
    } else {
        print!("{}[2J", 27 as char);
    }
  }
}