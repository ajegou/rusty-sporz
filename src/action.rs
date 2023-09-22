use serde::{Serialize, Deserialize};
use crate::{game::{PlayerGame, Game}, interface::Interface};

pub enum Action {
  UserAction(String, fn (&mut dyn PlayerGame, &mut Interface)),
  GeneralAction(String, fn (&mut dyn Game, &mut Interface)),
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum ActionType {
  Eliminate,
  Infect,
  Paralyze,
  Spy,
  Cure,
  Psychoanalyze,
  Genomyze,
}

pub fn get_menu_text(action: ActionType) -> String {
  match action {
    ActionType::Eliminate => String::from("Voter pour éliminer un·e de vos ami·e·s"),
    ActionType::Infect => String::from("Voter pour infecter un·e de ces sales humain·e·s"),
    ActionType::Paralyze => String::from("Voter pour paralyser un·e de ces sales humain·e·s"),
    ActionType::Spy => String::from("Surveiller un·e individu·e"),
    ActionType::Cure => String::from("Choisir un·e humain·e à soigner"),
    ActionType::Psychoanalyze => String::from("Choisir un·e client·e à psychanalyser"),
    ActionType::Genomyze => String::from("Choisir un génome à inspecter"),
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
    ActionType::Genomyze => String::from("Choisissez votre cobaye:"),
  }
}
