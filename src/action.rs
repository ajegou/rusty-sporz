use crate::game::GameStatus;

// #[derive(Debug)]
pub struct Action {
  pub description: String,
  pub execute: fn(&mut GameStatus),
}

impl PartialEq for Action {
  fn eq(&self, other: &Self) -> bool {
    return self.description == other.description;
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum ActionType {
  Eliminate,
  Infect,
  Paralyze,
  Spy,
  Cure,
  Psychoanalyze,
}

pub fn get_menu_text(action: ActionType) -> String {
  match action {
    ActionType::Eliminate => String::from("Voter pour éliminer un·e de vos ami·e·s"),
    ActionType::Infect => String::from("Voter pour infecter un·e de ces sales humain·e·s"),
    ActionType::Paralyze => String::from("Voter pour paralyser un·e de ces sales humain·e·s"),
    ActionType::Spy => String::from("Surveiller un·e individu·e"),
    ActionType::Cure => String::from("Choisir un·e humain·e à soigner"),
    ActionType::Psychoanalyze => String::from("Choisir un·e client·e à psychanalyser"),
  }
}

pub fn get_header_text(action: ActionType) -> String {
  match action {
    ActionType::Eliminate => String::from("Choisissez un·e camarade à éliminer:"),
    ActionType::Infect => String::from("Choisissez un·e humain·e à infecter:"),
    ActionType::Paralyze => String::from("Choisissez un·e humain·e à paralyser:"),
    ActionType::Spy => String::from("Choisissez qui vous allez stalker cette nuit:"),
    ActionType::Cure => String::from("Choisissez un·e humain·e à soigner:"),
    ActionType::Psychoanalyze => String::from("Choisissez votre client:"),
  }
}
