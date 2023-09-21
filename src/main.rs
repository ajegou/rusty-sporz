mod player;
mod action;
mod role;
mod game;
mod debug;
mod helper;
mod phases;
mod backup;
mod message;
mod interface;
mod game_creator;
use action::{Action, Action::{GeneralAction, UserAction}};
use debug::{mock_game_creator, mock_game_vote_tie};
use phases::{run_elimination_phase, run_it_phase, run_mutants_phase, run_physicians_phase, run_psychologist_phase};
use std::env;
use game::{ Game, PlayerGame, GameStatus };
use action::{ActionType, get_header_text, get_menu_text};
use role::Role;
use std::error;
use player::{Player, PlayerId};
use rand::Rng;

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

fn run_night(game: &mut dyn Game, interface: &mut Interface) {
  // Check that everyone played
  if !game.debug() {
    let living_players = game.get_alive_players();
    let missing_players = living_players
      .iter()
      .filter_map(|player| if player.has_connected_today { None } else { Some(&player.name) })
      .collect::<Vec<&String>>();
    if missing_players.len() > 0 {
      interface.user_validate(format!("J'exige la visite des membres d'équipages {:?} avant l'extinction des feux", missing_players).as_str());
      return;
    }
  }

  run_elimination_phase(interface, game);
  run_mutants_phase(game);
  run_physicians_phase(game);
  run_it_phase(game);
  run_psychologist_phase(game);

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

fn display_home_menu (game: &mut dyn Game, interface: &mut Interface) {
  interface.clear_terminal();
  println!("Bienvenue sur le terminal de control du {}", Color::Bright.color(game.get_name()));
  let mut actions_list: Vec<Action> = Vec::new();
  actions_list.push(GeneralAction(
    String::from("Identification"),
    run_action_log_in,
  ));
  actions_list.push(GeneralAction(
    String::from("Status de l'équipage"),
    run_action_crew_status,
  ));
  actions_list.push(GeneralAction(
    String::from("Fin de la journée"),
    run_night,
  ));
  match interface.user_select_action(&actions_list) {
    UserAction(_, _) => panic!(""), // Arghhhh, didn't expect to have to do this :/
    GeneralAction(_, run) => run(game, interface),
  }
}

fn run_action_log_in(game: &mut dyn Game, interface: &mut Interface) {
  interface.clear_terminal();
  let key = interface.user_non_empty_input("Entrez votre code d'identification:");
  let player_id = game.get_player_id_from_key(key);
  match player_id {
    Some(player_id) => {
      game.set_current_player_id(Some(player_id));
    }
    None => interface.user_validate("Code invalide, appuyez sur ENTREE pour revenir a l'écran d'accueil."),
  }
}

fn run_action_crew_status(game: &mut dyn Game, interface: &mut Interface) {
  let mut rng = rand::thread_rng(); // Used to generate random ids for display
  println!("\nStatus de l'équipage:");
  for player in game.get_all_players() {
    if game.debug() {
      println!("* Membre d'équipage n°{} - {} {}: {}",
        player.key,
        player.role,
        player.name,
        if player.alive {
          String::from(Color::FgGreen.color("Actif"))
        } else {
          format!("{} ({})", Color::Blink.color(Color::FgRed.color("Décédé·e").as_str()), player.get_death_cause())
        },
      )
    } else {
      println!("* Membre d'équipage n°{} - {}: {}",
        rng.gen_range(0..100),
        player.name,
        if player.alive {
          String::from(Color::FgGreen.color("Actif"))
        } else {
          format!("{} ({})", Color::Blink.color(Color::FgRed.color("Décédé·e").as_str()), player.get_death_cause())
        },
      )

    }
  }
  interface.user_validate("");
}

fn display_player_status_and_actions (game_status: &mut impl Game, interface: &mut Interface, current_player_id: PlayerId) {
  interface.clear_terminal();
  let game: &mut dyn PlayerGame = &mut game_status.get_player_game(current_player_id);
  game.get_mut_current_player().has_connected_today = true;
  let player = game.get_current_player();
  let mut actions_list = Vec::new();
  let status = if player.alive {
    if player.infected { Color::FgRed.color("mutant") } else { Color::FgGreen.color("saint")}
  } else {
    Color::Blink.color(Color::FgRed.color("Mort").as_str())
  };
  println!("Bienvenue {}, vous êtes un {} {}", player.name, player.role, status);
  if player.infected {
    println!("En tant que mutant, vous devez prendre le contrôle du vaisseau en infectant ou éliminant tous les membres d'équipage encore saints!");
  } else {
    println!("Vous devez nous aider à contenir la propagation et éliminer les mutants à bord avant qu'il ne soit trop tard!");
  }
  if player.messages.len() > 0 {
    println!("Messages personnels:");
    for message in &player.messages {
      println!("{}", message.to_string());
    }
  }

  add_exit_action(&mut actions_list);

  if player.alive {
    add_action_elimination(game, &mut actions_list);

    match game.get_current_player().role {
      Role::Patient0 => add_action_patient_0(game, &mut actions_list),
      Role::Psychologist => add_action_psychologist(game, &mut actions_list),
      Role::Physician => add_action_physician(game, &mut actions_list),
      Role::Geneticist => add_action_geneticist(game, &mut actions_list),
      Role::ITEngineer => add_action_it_engineer(game, &mut actions_list),
      Role::Spy => add_action_spy(game, &mut actions_list),
      Role::Hacker => add_action_hacker(game, &mut actions_list),
      Role::Traitor => add_action_traitor(game, &mut actions_list),
      Role::Astronaut => add_action_astronaut(game, &mut actions_list),
    }

    if game.get_current_player().infected {
      add_action_mutant(game, &mut actions_list);
    }
  }

  match interface.user_select_action(&actions_list) {
    UserAction(_, run) => run(game, interface),
    GeneralAction(_, run) => run(game_status, interface),
  }
}

fn log_out(game: &mut dyn Game, _interface: &mut Interface) {
  game.set_current_player_id(None);
}

fn end_game(game: impl Game, interface: &mut Interface) {
  interface.clear_terminal();

  let healthy_players = game.get_alive_players().iter().filter(|player| !player.infected).count();
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
// Action for elimination

fn add_action_elimination(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  add_target_action(
    game,
    actions_list,
    ActionType::Eliminate,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Eliminate),
  );
}

// Actions for mutants

fn add_action_mutant(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  add_target_action(
    game,
    actions_list,
    ActionType::Infect,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Infect),
  );
  add_target_action(
    game,
    actions_list,
    ActionType::Paralyze,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Paralyze),
  );
  // add kill
}

// Actions for roles

fn add_action_patient_0(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {}

fn add_action_psychologist(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  add_target_action(
    game,
    actions_list,
    ActionType::Psychoanalyze,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Psychoanalyze),
  );
}

fn add_action_physician(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  if !game.get_current_player().infected { // An infected physician cannot cure
    add_target_action(
      game,
      actions_list,
      ActionType::Cure,
      |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Cure),
    );
  }
}

fn add_action_geneticist(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {
  todo!();
}

fn add_action_it_engineer(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {}

fn add_action_spy(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  add_target_action(
    game,
    actions_list,
    ActionType::Spy,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Spy),
  );
}

fn add_action_hacker(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {
  todo!();
}

fn add_action_traitor(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {
  todo!();
}

fn add_action_astronaut(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {}

// Actions helpers

fn add_target_action(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>, action: ActionType, run: fn(&mut dyn PlayerGame, interface: &mut Interface)) {
  // It's a bit annoying to have to take "run" here, but closures using the scope seem to be a bit trickier
  actions_list.push(UserAction(
    match game.get_current_target(&action) {
      Some(target) => format!("{} [{}]", get_menu_text(action) , target.name),
      None => format!("{}", get_menu_text(action)),
    },
    run,
  ));
}

fn run_target_action(game: &mut dyn PlayerGame, interface: &mut Interface, action: ActionType) {
  interface.clear_terminal();
  match game.get_current_target(&action) {
    Some(target) => println!("{} [{}]", get_header_text(action), target.name),
    None => println!("{}", get_header_text(action)),
  }
  let targets: Vec<&Player> = game.get_alive_players();
  let selected = interface.user_select_target(&targets);
  game.set_current_target(&action, selected.map(|player| player.id));
}

// Selection helpers

fn add_exit_action(actions_list: &mut Vec<Action>) {
  actions_list.push(GeneralAction(String::from("Déconnection"), log_out ));
}
