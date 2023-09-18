mod player;
mod action;
mod role;
mod game;
mod debug;
mod message;
mod interface;
use interface::Color;
use std::env;
use std::fmt::format;
use std::slice::Iter;
use game::GameStatus;
use action::{Action,ActionType, get_header_text, get_menu_text};
use role::Role;
use std::error;
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::io::Error;
use rand::thread_rng;
use rand::prelude;
use player::Player;
use player::PlayerId;
use rand::seq::SliceRandom;

use crate::message::Message;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();
    let debug = args.contains(&String::from("--debug"));

    let players_names = get_players_list(debug)?;
    let number_of_players = u32::try_from(players_names.len()).unwrap();
    let mut roles = role::get_roles(number_of_players)?;
    roles.shuffle(&mut thread_rng());
    let (players, keys_to_ids) = create_players(players_names, roles);

    let mut game = GameStatus::new(players);
    game.debug = debug;
    start_game(game);

    return Ok(());
}

// Consumes both names_to_keys and roles
fn create_players<'a>(names_to_keys: HashMap<String, String>, mut roles: Vec<Role>)
    -> (Vec<Player>, HashMap<String, PlayerId>) {
    let mut next_user_id = 0;
    let mut players: Vec<Player> = Vec::new();
    let mut keys_to_ids: HashMap<String, PlayerId> = HashMap::new();
    for (name, key) in names_to_keys {
        keys_to_ids.insert(key.clone(), next_user_id);
        let role = roles.pop().unwrap();
        let player = Player::new(next_user_id, key, name, role);
        players.push(player);
        next_user_id += 1;
    }
    return (players, keys_to_ids);
}

fn get_players_list(use_debug: bool) -> Result<HashMap<String, String>, Error> {
    let mut keys: Vec<usize> = (100..1000).collect();
    if use_debug {
        keys.reverse(); // Want ids to be 100 101 102 ... in debug
    } else {
        keys.shuffle(&mut thread_rng());
    }

    let mut players: HashMap<String, String> = HashMap::new();
    loop {
        clear_terminal(None);
        let message = "Enter your name (or enter DONE if no more players):";
        let input;
        if use_debug {
            if players.len() == debug::NAMES.len() {
                break;
            }
            input = Some(debug::NAMES[players.len()].to_string());
        } else {
            input = query_and_validate(message)?;
        }
        match input {
            None => continue,
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
                        validate("");
                    }
                    players.insert(name, key);
                }
            }
        }
    }
    return Ok(players);
}

fn start_game (mut game: GameStatus) {
    if game.debug {
        run_action_crew_status(&mut game);
    }
    while !game.ended {
        match game.current_player_id {
            Some(_) => display_player_status_and_actions(&mut game),
            None => display_home_menu(&mut game),
        }
    }
}

fn run_night(game: &mut GameStatus) {
    let current_date = game.date.clone();

    // Check that everyone played
    if !game.debug {
        let living_players = game.get_alive_players();
        let missing_players = living_players
            .iter()
            .filter_map(|player| if player.has_connected_today { None } else { Some(&player.name) })
            .collect::<Vec<&String>>();
        if missing_players.len() > 0 {
            validate(format!("J'exige la visite des membres d'équipages {:?} avant l'extinction des feux", missing_players).as_str());
            return;
        }
    }

    // Notify the mutants of who the other mutants are
    // The newly converted mutant will only get that information at the next night
    let mutants_names = game.get_alive_players().iter()
        .filter_map(|player| if player.infected { Some(player.name.clone())} else { None })
        .collect::<Vec<String>>().join(" ");
    game.limited_broadcast(Message {
        date: current_date,
        source: String::from("Overmind"),
        content: String::from(format!("Lors du dernier crépuscule, les mutant·e·s étaient: [{mutants_names}]")),
    }, |player: &&mut &mut Player| player.infected);

    // Mutate one player
    let mutate_results = compute_votes_winner(
        game.get_alive_players().iter().filter(|player| player.infected),
        ActionType::Infect);
    if mutate_results.is_some() {
        let mutatee_name = &game.players[mutate_results.unwrap().0].name;
        game.limited_broadcast(Message { // Notify mutants of who was infected
            date: current_date,
            source: String::from("Overmind"),
            content: String::from(format!("Félicitations, cette nuit vous êtes parvenus à infecter: {mutatee_name}")),
        }, |player: &&mut &mut Player| player.infected);
        let mutate_winner = &mut game.players[mutate_results.unwrap().0];
        mutate_winner.infected = true;
        mutate_winner.send_message(Message { // Notify the new mutant that he was infected
            date: current_date,
            source: String::from("Overmind"),
            content: String::from(format!("Bienvenue {}, nous sommes heureuxe de vous compter parmis nous.", mutate_winner.name)),
        })
    }

    // Paralyze one player
    let paralyze_result = compute_votes_winner(
        game.get_alive_players().iter().filter(|player| player.infected), 
        ActionType::Paralyze);
    if paralyze_result.is_some() {
        let paralyzed_player = &mut game.players[paralyze_result.unwrap().0];
        paralyzed_player.paralyzed = true;
        paralyzed_player.messages.push(Message {
            date: current_date,
            source: String::from("Outil d'auto diagnostique"),
            content: String::from("Vous avez été paralysé pendant la nuit, vous n'avez donc pas pu faire d'action spéciale"),
        })
    }

    // Check votes to eliminate a player
    let elimination_results = compute_votes_results(
        game.get_alive_players().iter(),
        ActionType::Eliminate);
    let death_threshold = game.get_alive_players().len() / 2;
    for (target, votes) in elimination_results.iter() {
        if *votes > death_threshold {
            let player = &mut game.players[*target];
            player.alive = false;
            player.death_cause = Some(String::from("Aspiré·e accidentellement par le sas tribord"));
            
            let who_died = format!("Conformément à la volonté populaire, {} à été retiré du service actif.", player.name);
            let who_he_was;
            if player.infected {
                who_he_was = format!("L'autopsie à révélée que {} était en réalité un·e {} mutant·e!", player.name, player.role);
            } else {
                who_he_was = format!("{} était un·e honnête {} dévoué à la mission.", player.name, player.role);
            }
            let comment = "Vous pouvez lui dire adieu par le hublot tribord.";
            game.broadcast(Message {
                date: game.date,
                source: String::from("Ordinateur Central"),
                content: format!("{} {} {}", who_died, who_he_was, comment),
            })
        } else {
            let player = &mut game.players[*target];
            player.send_message(Message {
                date: game.date,
                source: String::from("Ordinateur Central"),
                content: format!("Cette nuit, {votes} membres d'équipages ont tenté de vous éliminer."),
            });
        }
    }

    // Tell the IT guy how many mutants are in play
    let infected_players = game.get_alive_players().iter().filter(|player| player.infected).count();
    for player in game.get_mut_alive_players() {
        if player.role == Role::ITEngineer && !player.paralyzed {
            player.send_message(Message {
                date: current_date,
                source: String::from("Système de diagnostique"),
                content: format!("L'analyse quantique de cette nuit a révélé la présence de {infected_players} membres d'équipage infectés à bord."),
            })
        }
    }

    // Cure one player
    game.prepare_new_turn();
}

// Returns the ID of the player who received the most votes for the given action among the voters, along the number of votes
// If several players received the highest number of votes, one is selected at random
fn compute_votes_winner <'a, T> (voters: T, action: ActionType) -> Option<(PlayerId, usize)>
where T: IntoIterator<Item = &'a&'a Player>,
{
    let results = compute_votes_results(voters, action);
    let mut winner: Option<usize> = None;
    let mut winner_votes = None;
    for (player, votes) in results.iter() {
        if winner_votes.is_none() || *votes > winner_votes.unwrap() {
            winner = Some(*player);
            winner_votes = Some(*votes);
        } else if *votes == winner_votes.unwrap() {
            if rand::random() { // Not great because if more than 2 "winners", the "first" one(s) is less likely to be selected
                winner = Some(*player);
                winner_votes = Some(*votes);
            }
        }
    }
    match winner {
        Some(player) => Some((player, winner_votes.unwrap())),
        None => None,
    }
}

fn compute_votes_results <'a, T> (voters: T, action: ActionType) -> HashMap<PlayerId, usize>
where T: IntoIterator<Item = &'a&'a Player>,
{
    let results = voters.into_iter().filter_map(|mutant| mutant.actions.get(&action))
    .fold(HashMap::new(), |mut acc, target| {
        *acc.entry(*target).or_insert(0) += 1;
        acc
    });
    return results;
}

fn display_home_menu (mut game: &mut GameStatus) {
    clear_terminal(Some(game));
    println!("Bienvenue sur le terminal de control du K-141 {}", Color::Bright.color("Koursk"));
    let mut actions_list = Vec::new();
    actions_list.push(Action {
        description: String::from("Identification"),
        execute: run_action_log_in,
    });
    actions_list.push(Action {
        description: String::from("Status de l'équipage"),
        execute: run_action_crew_status,
    });
    actions_list.push(Action {
        description: String::from("Fin de la journée"),
        execute: run_night,
    });
    let action = query_vote_actions(&actions_list);
    (action.execute)(&mut game);
}

fn run_action_log_in(game: &mut GameStatus) {
    clear_terminal(Some(game));
    let key = query_non_empty("Entrez votre code d'identification:");
    match game.players.iter().find(|player| player.key == key) {
        Some(player) => {
            game.current_player_id = Some(player.id);
            game.get_mut_current_player().has_connected_today = true;
        }
        None => validate("Code invalide, appuyez sur ENTREE pour revenir a l'écran d'accueil."),
    }
}

fn run_action_crew_status(game: &mut GameStatus) {
    println!("\nStatus de l'équipage:");
    for player in &game.players {
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
            println!("* Membre d'équipage - {}: {}",
                player.name,
                if player.alive {
                    String::from(Color::FgGreen.color("Actif"))
                } else {
                    format!("{} ({})", Color::Blink.color(Color::FgRed.color("Décédé·e").as_str()), player.get_death_cause())
                },
            )

        }
    }
    validate("");
}

fn display_player_status_and_actions (mut game: &mut GameStatus) {
    clear_terminal(Some(game));
    let player = game.get_current_player();
    let mut actions_list = Vec::new();
    println!("Bienvenue {}, vous êtes {}", player.name, player.role);
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

    add_action_elimination(&mut game, &mut actions_list);

    if game.get_current_player().paralyzed {
        println!("Vous avez été paralysé·e par les mutants!");
        println!("* vous ne pouvez donc utiliser vos capacités spéciales");
    } else {
        match game.get_current_player().role {
            Role::Patient0 => add_action_patient_0(&mut game, &mut actions_list),
            Role::Psychologist => add_action_psychologist(&mut game, &mut actions_list),
            Role::Physician => add_action_physician(&mut game, &mut actions_list),
            Role::Geneticist => add_action_geneticist(&mut game, &mut actions_list),
            Role::ITEngineer => add_action_it_engineer(&mut game, &mut actions_list),
            Role::Spy => add_action_spy(&mut game, &mut actions_list),
            Role::Hacker => add_action_hacker(&mut game, &mut actions_list),
            Role::Traitor => add_action_traitor(&mut game, &mut actions_list),
            Role::Astronaut => add_action_astronaut(&mut game, &mut actions_list),
        }
    }

    if game.get_current_player().infected {
        add_action_mutant(&mut game, &mut actions_list);
    }

    add_exit_action(&mut actions_list);

    let action = query_vote_actions(&actions_list);
    (action.execute)(&mut game);
}

fn log_out(game: &mut GameStatus) {
    game.current_player_id = None;
}

// Action for elimination

fn add_action_elimination(game: &mut GameStatus, actions_list: &mut Vec<Action>) {
    add_generic_action(
        game,
        actions_list,
        ActionType::Eliminate,
        |game: &mut GameStatus| run_generic_action(game, ActionType::Eliminate),
    );
}

// Actions for mutants

fn add_action_mutant(game: &mut GameStatus, actions_list: &mut Vec<Action>) {
    add_generic_action(
        game,
        actions_list,
        ActionType::Infect,
        |game: &mut GameStatus| run_generic_action(game, ActionType::Infect),
    );
    add_generic_action(
        game,
        actions_list,
        ActionType::Paralyze,
        |game: &mut GameStatus| run_generic_action(game, ActionType::Paralyze),
    );
    // add kill
}

// Actions for roles

fn add_action_patient_0(game: &mut GameStatus, actions_list: &mut Vec<Action>) {}

fn add_action_psychologist(game: &mut GameStatus, actions_list: &mut Vec<Action>) {
    add_generic_action(
        game,
        actions_list,
        ActionType::Psychoanalyze,
        |game: &mut GameStatus| run_generic_action(game, ActionType::Psychoanalyze),
    );
}

fn add_action_physician(game: &mut GameStatus, actions_list: &mut Vec<Action>) {
    if !game.get_current_player().infected {
        add_generic_action(
            game,
            actions_list,
            ActionType::Cure,
            |game: &mut GameStatus| run_generic_action(game, ActionType::Cure),
        );
    }
}

fn add_action_geneticist(game: &mut GameStatus, actions_list: &mut Vec<Action>) {
    todo!();
}

fn add_action_it_engineer(game: &mut GameStatus, actions_list: &mut Vec<Action>) {}

fn add_action_spy(game: &mut GameStatus, actions_list: &mut Vec<Action>) {
    add_generic_action(
        game,
        actions_list,
        ActionType::Spy,
        |game: &mut GameStatus| run_generic_action(game, ActionType::Spy),
    );
}

fn add_action_hacker(game: &mut GameStatus, actions_list: &mut Vec<Action>) {
    todo!();
}

fn add_action_traitor(game: &mut GameStatus, actions_list: &mut Vec<Action>) {
    todo!();
}

fn add_action_astronaut(game: &mut GameStatus, actions_list: &mut Vec<Action>) {}

// Actions helpers

fn add_generic_action(game: &mut GameStatus, actions_list: &mut Vec<Action>, action: ActionType, run: fn(&mut GameStatus)) {
    // It's a bit annoying to have to take "run" here, but closures using the scope seem to be a bit trickier
    actions_list.push(Action {
        description: match game.get_current_target(&action) {
            Some(target) => format!("{} [{}]", get_menu_text(action) , target.name),
            None => format!("{}", get_menu_text(action)),
        },
        execute: run,
    });
}

fn run_generic_action(game: &mut GameStatus, action: ActionType) {
    clear_terminal(Some(game));
    match game.get_current_target(&action) {
        Some(target) => println!("{} [{}]", get_header_text(action), target.name),
        None => println!("{}", get_header_text(action)),
    }
    let targets: Vec<&Player> = game.get_alive_players();
    let selected = query_targets(&targets);
    game.set_current_target(&action, selected.map(|player| player.id));
}

// Selection helpers

fn add_exit_action(actions_list: &mut Vec<Action>) {
    actions_list.push(Action { description: String::from("Déconnection"), execute: log_out });
}

fn query_targets<'a>(targets_list: &'a Vec<&'a Player>) -> Option<&'a Player> {
    for (idx, target) in targets_list.iter().enumerate() {
        println!("{idx}) {}", target.name);
    }
    println!("{}) {}", targets_list.len(), "Aucun");
    let accepted_answers: Vec<String> = (0..targets_list.len() + 1)
        .map(|value| { value.to_string() })
        .collect();
    let choice: usize = query_specific_answer("Quel est votre choix?", accepted_answers).parse().unwrap();
    if choice == targets_list.len() {
        return None;
    }
    return Some(targets_list[choice]);
}

fn query_vote_actions<'a>(actions_list: &'a Vec<Action>) -> &'a Action {
    for (idx, action) in actions_list.iter().enumerate() {
        println!("{idx}) {}", action.description);
    }
    let accepted_answers: Vec<String> = (0..actions_list.len())
        .map(|value| { value.to_string() })
        .collect();
    let choice: usize = query_specific_answer("Quel est votre choix?", accepted_answers).parse().unwrap();
    return &actions_list[choice];
}

fn query_specific_answer(message: &str, accepted_answers: Vec<String>) -> String {
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

fn validate(message: &str) {
    print!("{message} ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn query_non_empty(message: &str) -> String {
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

fn query_and_validate(message: &str) -> Result<Option<String>, io::Error> {
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

fn clear_terminal(game: Option<&GameStatus>) {
    if game.is_some_and(|game| game.debug) {
        print!("\n\n\n");
    } else {
        print!("{}[2J", 27 as char);
    }
}
