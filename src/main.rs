mod player;
mod action;
mod role;
mod game;
mod menu;
mod debug;
mod helper;
mod phases;
mod backup;
mod message;
mod interface;
mod game_creator;
use debug::{mock_game_creator, mock_game_vote_tie};
use menu::{display_player_status_and_actions, display_home_menu};
use phases::{run_elimination_phase, run_it_phase, run_mutants_phase, run_physicians_phase, run_psychologist_phase, run_spy_phase, run_geneticist_phase};
use std::{env, time::Duration};
use game::{ Game, GameStatus };
use std::error;

use crate::interface::{Interface, colors::Color};

fn main() -> Result<(), Box<dyn error::Error>> {
  let args: Vec<String> = env::args().collect();
  let debug = args.contains(&String::from("--debug"));

  let mut interface = Interface::new(debug);

  let mut game;
  if args.contains(&String::from("--from-backup")) {
    // Shitty but will do for now
    let mut iter = args.iter();
    while iter.next().unwrap() != &String::from("--from-backup") {}
    let path = iter.next().unwrap();
    game = GameStatus::restore_from_backup(path).unwrap();
  } else {
    if debug {
      mock_game_creator(&mut interface);
    }

    game = game_creator::create_game(&mut interface, debug)?;

    if debug {
      mock_game_vote_tie(&mut interface, &mut game);
    }
  }

  start_game(game, &mut interface);

  return Ok(());
}

fn start_game (mut game: impl Game, interface: &mut Interface) {
  while !game.ended() {
    match game.get_current_player_id() {
      Some(current_player_id) => {
        display_player_status_and_actions(
          &mut game,
          interface,
          current_player_id,
        );
      }
      None => display_home_menu(&mut game, interface),
    }
  }
  end_game(game, interface);
}

pub fn run_end_of_day (game: &mut dyn Game, interface: &mut Interface) {
  // Check that everyone played
  if !game.debug() {
    let living_players = game.get_players();
    let missing_players = living_players
      .iter()
      .filter_map(|player| if player.has_connected_today { None } else { Some(&player.name) })
      .collect::<Vec<&String>>();
    if missing_players.len() > 0 {
      interface.user_validate(format!("J'exige la visite des membres d'équipages {:?} avant l'extinction des feux", missing_players).as_str());
      return;
    }
  }

  let victim = run_elimination_phase(interface, game);
  game.set_phase_of_day(game::PhaseOfDay::Twilight); // not sure who should control this
  if victim.is_none() { // If no-one died we directly play the night
    run_night(game, interface);
  }
}

pub fn run_night(game: &mut dyn Game, interface: &mut Interface) {

  let killed_by_mutants = run_mutants_phase(game);
  let killed_by_physicians = run_physicians_phase(game);
  run_it_phase(game);
  run_psychologist_phase(game);
  run_geneticist_phase(game);
  run_spy_phase(game);

  interface.clear_terminal();
  interface.wait_and_display("La nuit passe...", Duration::from_secs(5), Duration::from_millis(700));

  if killed_by_mutants == None && killed_by_physicians == None {
    interface.play_no_death_good_sound();
    interface.user_validate("Rien à signaler pour cette nuit");
  } else {
    let mut dead_players = Vec::new();
    if let Some(player) = killed_by_mutants {
      dead_players.push(game.get_player(player).name.clone());
    }
    if let Some(player) = killed_by_physicians {
      dead_players.push(game.get_player(player).name.clone());
    }
    interface.play_death_sound();
    interface.user_validate(format!("C'est avec tristesse et amertume que nous vous annonçons la perte accidentelle de {} cette nuit",
      dead_players.join(" et ")).as_str());
  }

  game.prepare_new_turn();

  backup(game, interface);
}

fn backup (game: &mut dyn Game, interface: &mut Interface) {
  if let Err(error) = game.backup("backups/") {
    interface.clear_terminal();
    println!("WARNING - Backup Error: details written to stderr");
    eprintln!("WARNING - Backup Error: {}", error);
    interface.user_validate("Appuyez sur entrée pour continuer");
    interface.clear_terminal();
  }
}

fn end_game(game: impl Game, interface: &mut Interface) {
  interface.clear_terminal();

  let healthy_players = game.get_players().iter().filter(|player| !player.infected).count();
  if healthy_players == 0 {
    println!("===== Victoire des mutants =====");
    println!("Le {} est maintenant aux mains des mutants et, avec la coopération des centaines de passagers en sommeil, essaimera la mutation dans la galaxie.", Color::Bright.color(game.get_name()));
    println!("Féliciations aux mutants");
    println!("Vous êtes l'avenir de l'humanité");
    println!("Mais il reste beaucoup à faire...");
  } else {
    println!("===== Victoire de l'humanité =====");
    println!("L'équipage du {} est parvenu, au prix de grands sacrifices, à contenir et éliminer la mutation.", Color::Bright.color(game.get_name()));
    println!("Féliciations aux survivants");
    println!("Grâce à vous l'humanité est sauve");
    println!("Pour le moment...");
  }
}
