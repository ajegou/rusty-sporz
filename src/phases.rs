use crate::{
  game::{GameStatus, Game},
  role::Role,
  message::Message,
  action::ActionType,
  helper::{compute_votes_results, compute_votes_winner},
  player::Player};


pub fn run_mutants_phase(game: &mut GameStatus) {
  let current_date = game.get_date(); // do better

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
    if let Some((player_id, _)) = mutate_results {
        let mutatee_name = &game.get_player(player_id).name;
        game.limited_broadcast(Message { // Notify mutants of who was infected
            date: current_date,
            source: String::from("Overmind"),
            content: String::from(format!("Félicitations, cette nuit vous êtes parvenus à infecter: {mutatee_name}")),
        }, |player: &&mut &mut Player| player.infected);
        let mutate_winner = game.get_mut_player(player_id);
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
    if let Some((player_id, _)) = paralyze_result {
        let paralyzed_player = game.get_mut_player(player_id);
        paralyzed_player.paralyzed = true;
        paralyzed_player.messages.push(Message {
            date: current_date,
            source: String::from("Outil d'auto diagnostique"),
            content: String::from("Vous avez été paralysé pendant la nuit, vous n'avez donc pas pu faire d'action spéciale"),
        })
    }
}


pub fn run_physicians_phase(game: &mut GameStatus) {
  let current_date = game.get_date(); // do better

  // Cure one player
  let mut alive_players = game.get_mut_alive_players();
  let physicians = alive_players
    .iter_mut()
    .filter(|player| player.role == Role::Physician);
  let mut cured_players = Vec::new();
  let mut active_physicians = Vec::new();
  let mut active_physician_names = Vec::new();
  for physician in physicians {
    if physician.infected || physician.paralyzed {
      physician.send_message(Message {
        date: current_date,
        source: String::from("Outil d'auto diagnostique"),
        content: if physician.infected {
          String::from("Vous êtes avez été infecté·e pendant la nuit, vous n'avez donc pu participer aux soins")
        } else {
          String::from("Vous avez été·e paralysé·e pendant la nuit, vous n'avez donc pu participer aux soins")
        }
      });
    } else {
      if let Some(target) = physician.actions.get(&ActionType::Cure) {
        cured_players.push(*target);
      }
      active_physicians.push(physician.id);
      active_physician_names.push(physician.name.clone());
    }
  }

  let mut cured_players_names = Vec::new();
  for cured_player in cured_players { // Send messages to the active medical team about who was cured
    cured_players_names.push(game.get_player(cured_player).name.clone());
    if game.get_player(cured_player).role == Role::Patient0 {
      game.get_mut_player(cured_player).send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez subit un traitement par irradiation intense cette nuit, mais la mutation est trop avancée chez vous, cela a échoué"),
      });
    } else if game.get_player(cured_player).infected {
      game.get_mut_player(cured_player).infected = false;
      game.get_mut_player(cured_player).send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez subit un traitement par irradiation intense cette nuit, qui vous à débarrassé de toute trace de mutation"),
      });
    } else {
      game.get_mut_player(cured_player).send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez subit un traitement anti-mutation cette nuit, bien qu'il n'y ait eu aucune trace de mutations dans votre corps"),
      });
    }
  }
  let active_physician_names = active_physician_names.join(" ");
  let cured_players_names = cured_players_names.join(" ");
  for active_physician in active_physicians {
    game.get_mut_player(active_physician).send_message(Message {
      date: current_date,
      source: String::from("Équipe médicale"),
      content: String::from(format!("L'équipe médicale opérationelle de la nuit précédente ({}) est parvenue à soigner: [{}]", active_physician_names, cured_players_names)),
    });
  }
}

pub fn run_elimination_phase(game: &mut GameStatus) {
  let current_date = game.get_date(); // do better

  // Check votes to eliminate a player
  let elimination_results = compute_votes_results(
    game.get_alive_players().iter(),
    ActionType::Eliminate);
  let death_threshold = game.get_alive_players().len() / 2;
  for (target, votes) in elimination_results.iter() {
    if *votes > death_threshold {
      let player = game.get_mut_player(*target);
      player.alive = false;
      player.death_cause = Some(String::from("Aspiré·e accidentellement par le sas tribord"));
      
      let who_died = format!("Conformément à la volonté populaire, {} à été retiré du service actif.", player.name);
      let who_he_was;
      if player.infected {
        let role = if player.role == Role::Patient0 { &Role::Astronaut } else { &player.role }; // Patient0's is not revealed on death
        who_he_was = format!("L'autopsie à révélée que {} était en réalité un·e {} mutant·e!", player.name, role);
      } else {
        who_he_was = format!("{} était un·e honnête {} dévoué à la mission.", player.name, player.role);
      }
      let comment = "Vous pouvez lui dire adieu par le hublot tribord.";
      game.broadcast(Message {
        date: current_date,
        source: String::from("Ordinateur Central"),
        content: format!("{} {} {}", who_died, who_he_was, comment),
      })
    } else {
      let player = game.get_mut_player(*target);
      player.send_message(Message {
        date: current_date,
        source: String::from("Ordinateur Central"),
        content: format!("Cette nuit, {votes} membres d'équipages ont tenté de vous éliminer."),
      });
    }
  }
}

pub fn run_it_phase(game: &mut GameStatus) {
  let current_date = game.get_date(); // do better

  // Tell the IT guy how many mutants are in play
  let infected_players = game.get_alive_players().iter().filter(|player| player.infected).count();
  for player in game.get_mut_alive_players() {
    if player.role == Role::ITEngineer && !player.paralyzed {
      player.send_message(Message {
        date: current_date,
        source: String::from("Système de diagnostique"),
        content: format!("L'analyse quantique de cette nuit a révélé la présence de {infected_players} membres d'équipage infectés à bord."),
      })
    } // See if we want to display something in else
  }
}

pub fn run_psychologist_phase(game: &mut GameStatus) {
  let current_date = game.get_date(); // do better
  let psychologists_ids = game.get_player_ids(|player| player.role == Role::Psychologist);
  for psychologists_id in psychologists_ids {
    if !game.get_player(psychologists_id).paralyzed {
      if let Some(target) = game.get_player(psychologists_id).get_target(&ActionType::Psychoanalyze) {
        let name = game.get_player(*target).name.clone();
        if game.get_player(*target).infected {
          game.get_mut_player(psychologists_id).send_message(Message {
            date: current_date,
            source: String::from("Freud GPT"),
            content: format!("D'après l'analyse, il semblerait que le comportement déviant de {} ne découle pas d'un trauma d'enfance, mais d'un changement récent. C'est un·e mutant·e!", name),
          })
        } else {
          game.get_mut_player(psychologists_id).send_message(Message {
            date: current_date,
            source: String::from("Freud GPT"),
            content: format!("D'après l'analyse, il semblerait que le comportement déviant de {} découle simplement d'un rapport difficile à la mère, et pas d'une mutation génétique", name),
          })
        }
      }
    } // See if we want to display something in else
  }
}
