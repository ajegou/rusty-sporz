use std::fmt;

#[derive(PartialEq, Debug)]
pub enum Role {
  Patient0,
  Psychologist,
  Physician,
  Geneticist,
  ITEngineer,
  Spy,
  Hacker,
  Traitor,
  Astronaut,
}

impl fmt::Display for Role {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
        Role::Patient0 => write!(f, "Patient·e 0"),
        Role::Psychologist => write!(f, "Psychologue"),
        Role::Physician => write!(f, "Médecin"),
        Role::Geneticist => write!(f, "Geneticien·ne"),
        Role::ITEngineer => write!(f, "Informaticien·ne"),
        Role::Spy => write!(f, "Espion·ne"),
        Role::Hacker => write!(f, "Hacker·euse"),
        Role::Traitor => write!(f, "Traitre·sse"),
        Role::Astronaut => write!(f, "Astronaute"),
    }
}
}

pub fn get_roles (number_of_players: u32) -> Result<Vec<Role>, String>{ // maybe return specific errors
  if number_of_players < 7 {
    return Err("Sorry, the game cannot be played with less than 7 players.".to_string());
  }
  if number_of_players > 7 {
    return Err("Sorry, the game cannot currently be played with more than 7 players.".to_string());
  }
  return Ok(vec![
    Role::Patient0,
    Role::Psychologist,
    Role::ITEngineer,
    Role::Physician,
    Role::Physician,
    Role::Astronaut,
    Role::Spy,
  ]);
}