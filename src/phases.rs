use crate::{game::GameStatus, role::Role, message::Message, action::ActionType, helper::{compute_votes_results, compute_votes_winner}, player::Player};


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
  for cured_player in cured_players {
    if game.players[cured_player].role == Role::Patient0 {
      game.players[cured_player].send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez subit un traitement par irradiation intense cette nuit, mais la mutation est trop avancée chez vous, cela a échoué"),
      });
    } else if game.players[cured_player].infected {
      game.players[cured_player].infected = false;
      game.players[cured_player].send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez subit un traitement par irradiation intense cette nuit, qui vous à débarrassé de toute trace de mutation"),
      });
    } else {
      game.players[cured_player].send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez subit un traitement anti-mutation cette nuit, bien qu'il n'y ait eu aucune trace de mutations dans votre corps"),
      });
    }
  }
  let active_physician_names = active_physician_names.join(" ");
  for active_physician in active_physicians {
    game.players[active_physician].send_message(Message {
      date: current_date,
      source: String::from("Équipe médicale"),
      content: String::from(format!("Liste de l'équipe médicale opérationelle la nuit dernière: [{}]", active_physician_names)),
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
        date: current_date,
        source: String::from("Ordinateur Central"),
        content: format!("{} {} {}", who_died, who_he_was, comment),
      })
    } else {
      let player = &mut game.players[*target];
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
    }
  }
}
