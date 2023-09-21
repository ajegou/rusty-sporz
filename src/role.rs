use std::fmt;
use serde::{Serialize, Deserialize};

#[allow(dead_code)]
#[derive(PartialEq, Debug, Serialize, Deserialize)]
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
