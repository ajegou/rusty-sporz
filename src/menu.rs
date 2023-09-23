use crate::{game::{Game, PlayerGame, PhaseOfDay}, interface::{Interface, colors::Color}, action::{Action, Action::{GeneralAction, UserAction}, ActionType, get_header_text, get_menu_text}, player::{Player, PlayerId}, role::Role, run_night, run_end_of_day};

use rand::Rng;

pub fn display_home_menu (game: &mut dyn Game, interface: &mut Interface) {
  interface.clear_terminal();
  println!("Bienvenue sur le terminal de control du {}", Color::Bright.color(game.get_name()));
  match game.get_phase_of_day() {
    PhaseOfDay::Day => println!("Nous sommes le {}ème jour après détection de l'infection", game.get_date()),
    PhaseOfDay::Twilight => println!("Nous sommes au crépuscule du {}ème jour après détection de l'infection", game.get_date()),
}
  let mut actions_list: Vec<Action> = Vec::new();
  actions_list.push(GeneralAction(
    String::from("Identification"),
    run_action_log_in,
  ));
  actions_list.push(GeneralAction(
    String::from("Status de l'équipage"),
    run_action_crew_status,
  ));
  match game.get_phase_of_day() {
    PhaseOfDay::Day => actions_list.push(GeneralAction(
      String::from("Fin de la journée"),
      run_end_of_day,
    )),
    PhaseOfDay::Twilight => actions_list.push(GeneralAction(
      String::from("Passer au jour suivant"),
      run_night,
    )),
  }
  match interface.user_select_action(&actions_list) {
    UserAction(_, _) => panic!(""), // Arghhhh, didn't expect to have to do this :/
    GeneralAction(_, run) => run(game, interface),
  }
}

pub fn run_action_log_in(game: &mut dyn Game, interface: &mut Interface) {
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

pub fn run_action_crew_status(game: &mut dyn Game, interface: &mut Interface) {
  let mut rng = rand::thread_rng(); // Used to generate random ids for display
  println!("\nStatus de l'équipage:");
  for player in game.get_all_players() {
    if game.debug() {
      println!("* Membre d'équipage n°{} - {} {}{} {}: {}",
        player.key,
        player.role,
        if player.infected { Color::FgRed.color("mutant") } else { Color::FgGreen.color("saint") },
        if player.host {
          String::from(" (hôte)")
        } else if player.resilient {
          String::from(" (resistant)")
        } else {
          String::from("")
        },
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

pub fn display_player_status_and_actions (game_status: &mut impl Game, interface: &mut Interface, current_player_id: PlayerId) {
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
  if player.role == Role::Physician { //Physicians know the list of other physicians
    let physician_names: Vec<String> = game.get_players().iter()
      .filter_map(|player| if player.role == Role::Physician { Some(player.name.clone()) } else { None }).collect();
    println!("* Membres de l'équipe médicale: [{}]", physician_names.join(", "));
  }
  if player.infected {
    println!("En tant que mutant, vous devez prendre le contrôle du vaisseau en infectant ou éliminant tous les membres d'équipage encore saints!");
  } else {
    println!("Vous devez nous aider à contenir la propagation et éliminer les mutants à bord avant qu'il ne soit trop tard!");
  }
  if player.messages.len() > 0 {
    println!("");
    println!("Messages personnels:");
    for message in &player.messages {
      println!("{}", message.to_string());
    }
  }

  add_log_out_action(&mut actions_list);

  if player.alive {
    if game.get_phase_of_day() == &PhaseOfDay::Day {
      // Players are only allowed to vote during the day
      add_action_elimination(game, &mut actions_list);
    } else {
      println!("");
      print!("Le vote sur l'élimination d'un membre d'équipage à déjà au lieu pour aujoud'hui");
      println!(" (revenez demain pour une autre chance d'assassiner un de vos amis!)");
    }

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

  println!("");
  match interface.user_select_action(&actions_list) {
    UserAction(_, run) => run(game, interface),
    GeneralAction(_, run) => run(game_status, interface),
  }
}

// Action for elimination

pub fn add_action_elimination(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  add_target_action(
    game,
    actions_list,
    ActionType::Eliminate,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Eliminate),
  );
}

// Actions for mutants

pub fn add_action_mutant(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
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

pub fn add_action_patient_0(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {}

pub fn add_action_psychologist(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  add_target_action(
    game,
    actions_list,
    ActionType::Psychoanalyze,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Psychoanalyze),
  );
}

pub fn add_action_physician(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  if !game.get_current_player().infected { // An infected physician cannot cure
    add_target_action( // Action to select some to cure
      game,
      actions_list,
      ActionType::Cure,
      |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Cure),
    );
    actions_list.push(Action::UserAction( // Action to toggle auto-cure of other physicians
      if game.get_current_player().auto_cure_physician {
        format!("Désactiver le soin automatique des autres médecins [{}]", Color::FgGreen.color("Activé"))
      } else {
        format!("Activer le soin automatique des autres médecins [{}]", Color::FgRed.color("Désactivé"))
      },
      |game: &mut dyn PlayerGame, _interface: &mut Interface| {
        let current_player = game.get_mut_current_player();
        current_player.auto_cure_physician = !current_player.auto_cure_physician;
      }
    ));
  }
}

pub fn add_action_geneticist(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  add_target_action(
    game,
    actions_list,
    ActionType::Genomyze,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Genomyze),
  );
}

pub fn add_action_it_engineer(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {}

pub fn add_action_spy(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  add_target_action(
    game,
    actions_list,
    ActionType::Spy,
    |game: &mut dyn PlayerGame, interface: &mut Interface| run_target_action(game, interface, ActionType::Spy),
  );
}

pub fn add_action_hacker(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
  actions_list.push(Action::UserAction( // Action to toggle auto-cure of other physicians
    match game.get_current_player().hacker_target {
      Some(target) => format!("Selectionner un role à pirater [{}]", target),
      None => format!("Selectionner un role à pirater"),
    },
    |game: &mut dyn PlayerGame, interface: &mut Interface| {
      let hackable_roles = vec![Role::Geneticist, Role::ITEngineer, Role::Spy];
      let hackable_roles = game.get_players().iter()
        .filter_map(|player| if hackable_roles.contains(&player.role) { Some(player.role) } else { None })
        .collect::<Vec<Role>>();
      if hackable_roles.len() == 0 {
        interface.user_validate("Désolé, il n'y a personne que vous puissiez hacker");
      } else {
        game.get_mut_current_player().hacker_target = Some(*interface.user_select_from(hackable_roles.iter()));
      }
    }
  ));
}

pub fn add_action_traitor(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {
  todo!();
}

pub fn add_action_astronaut(_game: &mut dyn PlayerGame, _actions_list: &mut Vec<Action>) {}

// Actions helpers

pub fn add_target_action(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>, action: ActionType, run: fn(&mut dyn PlayerGame, interface: &mut Interface)) {
  // It's a bit annoying to have to take "run" here, but closures using the scope seem to be a bit trickier
  actions_list.push(Action::UserAction(
    match game.get_current_target(&action) {
      Some(target) => format!("{} [{}]", get_menu_text(action) , target.name),
      None => format!("{}", get_menu_text(action)),
    },
    run,
  ));
}

pub fn run_target_action(game: &mut dyn PlayerGame, interface: &mut Interface, action: ActionType) {
  interface.clear_terminal();
  match game.get_current_target(&action) {
    Some(target) => println!("{} [{}]", get_header_text(action), target.name),
    None => println!("{}", get_header_text(action)),
  }
  let targets: Vec<&Player> = game.get_players();
  let selected = interface.user_select_target(&targets);
  game.set_current_target(&action, selected.map(|player| player.id));
}

// Selection helpers

pub fn add_log_out_action(actions_list: &mut Vec<Action>) {
  actions_list.push(Action::GeneralAction(String::from("Déconnection"), run_log_out ));
}

pub fn run_log_out(game: &mut dyn Game, _interface: &mut Interface) {
  game.set_current_player_id(None);
}
