mod player;
mod action;
mod role;
mod game;
mod debug;
mod helper;
mod phases;
mod message;
mod interface;
use action::{Action, Action::{GeneralAction, UserAction}};
use interface::Color;
use phases::{run_elimination_phase, run_it_phase, run_mutants_phase, run_physicians_phase, run_psychologist_phase};
use std::env;
use game::{ GameStatus, Game, PlayerGame };
use action::{ActionType, get_header_text, get_menu_text};
use role::Role;
use std::error;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::io::Error;
use rand::thread_rng;
use player::{Player, PlayerId};
use rand::seq::SliceRandom;
use rand::Rng;

static mut DEBUG: bool = false;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();
    let debug = args.contains(&String::from("--debug"));
    unsafe { DEBUG = debug }; // Shitty, mostly for interface, will do better

    let players_names = get_players_list(debug)?;
    let number_of_players = u32::try_from(players_names.len()).unwrap();
    let mut roles = role::get_roles(number_of_players)?;
    roles.shuffle(&mut thread_rng());
    let players = create_players(players_names, roles);

    let mut game = GameStatus::new(players);
    game.debug = debug;
    start_game(game);

    return Ok(());
}

// Consumes both names_to_keys and roles
fn create_players<'a>(names_to_keys: HashMap<String, String>, mut roles: Vec<Role>) -> Vec<Player> {
    let mut next_user_id = 0;
    let mut players: Vec<Player> = Vec::new();
    for (name, key) in names_to_keys {
        let role = roles.pop().unwrap();
        let player = Player::new(next_user_id, key, name, role);
        players.push(player);
        next_user_id += 1;
    }
    return players;
}

fn get_players_list(use_debug: bool) -> Result<HashMap<String, String>, Error> {
    let mut keys: Vec<usize> = (100..1000).collect();
    if use_debug {
        keys.reverse(); // Want ids to be 100 101 102 ... in debug
    } else {
        keys.shuffle(&mut thread_rng());
    }

    clear_terminal();
    let mut players: HashMap<String, String> = HashMap::new();
    loop {
        let message = "Enter your name (or enter DONE if no more players):";
        let input;
        if use_debug {
            if players.len() == debug::NAMES.len() {
                break;
            }
            input = Some(debug::NAMES[players.len()].to_string());
        } else {
            input = user_ask_and_validate(message)?;
        }
        match input {
            None => println!("Annulation"),
            Some(name) => {
                if name == "DONE" {
                    break;
                }
                if players.contains_key(&name) {
                    println!("Name '{name}' is already in use");
                } else {
                    let key = keys.pop().unwrap().to_string();
                    println!("Your secret key is {key}, do not forget it! You will need it to log-in later");
                    if !use_debug {
                        user_validate("");
                    }
                    players.insert(name, key);
                    clear_terminal();
                }
            }
        }
    }
    return Ok(players);
}

fn start_game (mut game: GameStatus) {
    while !&game.ended {
        match game.current_player_id {
            Some(current_player_id) => {
                display_player_status_and_actions(
                    &mut game,
                    current_player_id,
                );
            }
            None => display_home_menu(&mut game),
        }
    }
    end_game(&game);
}

fn run_night(game: &mut GameStatus) {
    // Check that everyone played
    if !game.debug {
        let living_players = game.get_alive_players();
        let missing_players = living_players
            .iter()
            .filter_map(|player| if player.has_connected_today { None } else { Some(&player.name) })
            .collect::<Vec<&String>>();
        if missing_players.len() > 0 {
            user_validate(format!("J'exige la visite des membres d'équipages {:?} avant l'extinction des feux", missing_players).as_str());
            return;
        }
    }

    run_mutants_phase(game);
    run_physicians_phase(game);
    run_elimination_phase(game);
    run_it_phase(game);
    run_psychologist_phase(game);

    let healthy_players = game.get_alive_players().iter().filter(|player| !player.infected).count();
    if healthy_players == 0 || healthy_players == game.get_alive_players().len() {
        game.ended = true;
    }

    game.prepare_new_turn();
}

fn display_home_menu (mut game: &mut GameStatus) {
    clear_terminal();
    if game.debug {
        run_action_crew_status(&mut game);
    }
    println!("Bienvenue sur le terminal de control du K-141 {}", Color::Bright.color("Koursk"));
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
    match user_select_action(&actions_list) {
        UserAction(_, _) => panic!(""), // Arghhhh, didn't expect to have to do this :/
        GeneralAction(_, run) => run(&mut game),
    }
}

fn run_action_log_in(game: &mut GameStatus) {
    clear_terminal();
    let key = user_non_empty_input("Entrez votre code d'identification:");
    let player_id = game.get_player_id_from_key(key);
    match player_id {
        Some(player_id) => {
            game.current_player_id = Some(player_id);
        }
        None => user_validate("Code invalide, appuyez sur ENTREE pour revenir a l'écran d'accueil."),
    }
}

fn run_action_crew_status(game: &mut GameStatus) {
    let mut rng = rand::thread_rng(); // Used to generate random ids for display
    println!("\nStatus de l'équipage:");
    for player in game.get_all_players() {
        if game.debug {
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
    user_validate("");
}

fn display_player_status_and_actions (mut game_status: &mut GameStatus, current_player_id: PlayerId) {
    clear_terminal();
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

    add_exit_action(&mut actions_list);

    match user_select_action(&actions_list) {
        UserAction(_, run) => run(game),
        GeneralAction(_, run) => run(&mut game_status),
    }
}

fn log_out(game: &mut GameStatus) {
    game.current_player_id = None;
}

fn end_game(game: &GameStatus) {
    clear_terminal();

    let healthy_players = game.get_alive_players().iter().filter(|player| !player.infected).count();
    if healthy_players == 0 {
        println!("===== Victoire des mutants =====");
        println!("Le Koursk est maintenant aux mains des mutants et, avec la coopération des centaines de passagers en sommeil, essaimera la mutation dans la galaxie.");
        println!("Féliciations aux mutants");
        println!("Vous êtes l'avenir de l'humanité");
        println!("Mais il reste beaucoup à faire...");
    } else {
        println!("===== Victoire de l'humanité =====");
        println!("L'équipage du Koursk est parvenu, au prix de grands sacrifices, à contenir et éliminer la mutation.");
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
        |game: &mut dyn PlayerGame| run_target_action(game, ActionType::Eliminate),
    );
}

// Actions for mutants

fn add_action_mutant(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
    add_target_action(
        game,
        actions_list,
        ActionType::Infect,
        |game: &mut dyn PlayerGame| run_target_action(game, ActionType::Infect),
    );
    add_target_action(
        game,
        actions_list,
        ActionType::Paralyze,
        |game: &mut dyn PlayerGame| run_target_action(game, ActionType::Paralyze),
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
        |game: &mut dyn PlayerGame| run_target_action(game, ActionType::Psychoanalyze),
    );
}

fn add_action_physician(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>) {
    if !game.get_current_player().infected { // An infected physician cannot cure
        add_target_action(
            game,
            actions_list,
            ActionType::Cure,
            |game: &mut dyn PlayerGame| run_target_action(game, ActionType::Cure),
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
        |game: &mut dyn PlayerGame| run_target_action(game, ActionType::Spy),
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

fn add_target_action(game: &mut dyn PlayerGame, actions_list: &mut Vec<Action>, action: ActionType, run: fn(&mut dyn PlayerGame)) {
    // It's a bit annoying to have to take "run" here, but closures using the scope seem to be a bit trickier
    actions_list.push(UserAction(
        match game.get_current_target(&action) {
            Some(target) => format!("{} [{}]", get_menu_text(action) , target.name),
            None => format!("{}", get_menu_text(action)),
        },
        run,
    ));
}

fn run_target_action(game: &mut dyn PlayerGame, action: ActionType) {
    clear_terminal();
    match game.get_current_target(&action) {
        Some(target) => println!("{} [{}]", get_header_text(action), target.name),
        None => println!("{}", get_header_text(action)),
    }
    let targets: Vec<&Player> = game.get_alive_players();
    let selected = user_select_target(&targets);
    game.set_current_target(&action, selected.map(|player| player.id));
}

// Selection helpers

fn add_exit_action(actions_list: &mut Vec<Action>) {
    actions_list.push(GeneralAction(String::from("Déconnection"), log_out ));
}

fn user_select_target<'a>(targets_list: &'a Vec<&'a Player>) -> Option<&'a Player> {
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

fn user_select_action<'a>(actions_list: &'a Vec<Action>) -> &'a Action {
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

fn user_validate(message: &str) {
    print!("{message} ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn user_non_empty_input(message: &str) -> String {
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

fn user_ask_and_validate(message: &str) -> Result<Option<String>, io::Error> {
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

fn clear_terminal() {
    if unsafe { DEBUG } { // do better
        print!("\n\n\n");
    } else {
        print!("{}[2J", 27 as char);
    }
}
