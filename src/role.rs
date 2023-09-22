use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Role {
  Patient0,
  Physician,
  Psychologist,
  ITEngineer,
  Spy,
  Geneticist,
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
