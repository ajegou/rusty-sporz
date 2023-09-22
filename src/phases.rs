use crate::{
  game::Game,
  role::Role,
  message::Message,
  action::ActionType,
  helper::{compute_votes_results, compute_votes_winner},
  player::{Player, PlayerId}, interface::{Interface, colors::Color}};


pub fn run_elimination_phase(interface: &mut Interface, game: &mut dyn Game) {
  let current_date = game.get_date(); // do better

  // Check votes to eliminate a player
  let elimination_results = compute_votes_results(
    game.get_players().iter(),
    ActionType::Eliminate);
  let mut number_of_votes: Vec<usize> = elimination_results.values().map(|count|*count).collect();

  let white_votes: usize = game.get_players().len() - number_of_votes.iter().sum::<usize>();
  number_of_votes.push(white_votes);
  let max_number_of_votes = number_of_votes.iter().max().unwrap(); // cannot be empty

  let mut players_with_max_number: Vec<Option<PlayerId>> = elimination_results.iter()
    .filter_map(|(player, votes)| {
      if votes == max_number_of_votes { Some(player) } else { None }
    }).map(|player| Some(*player)) // So we can add None for the whites
    .collect();
  if *max_number_of_votes == white_votes {
    players_with_max_number.push(None);
  }

  // Notify everyone of how many crew members attempted to kill you, if any
  for (target, votes) in elimination_results.iter() {
    let player = game.get_mut_player(*target);
    player.send_message(Message {
      date: current_date,
      source: String::from("Ordinateur Central"),
      content: format!("Cette nuit, {votes} membres d'équipages ont tenté de vous éliminer."),
    });
  }

  let dead_crew_member = select_who_dies(interface, game, players_with_max_number);
  match dead_crew_member {
    Some(player) => {
      let player = game.get_mut_player(player);
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
    },
    None => game.broadcast(Message {
      date: current_date,
      source: String::from("Ordinateur Central"),
      content: String::from("Tout le monde a très bien dormi cette nuit."),
    }),
  }
}

fn select_who_dies (interface: &mut Interface, game: &dyn Game, options: Vec<Option<PlayerId>>) -> Option<PlayerId> {
  if options.len() == 0 {
    return None;
  }
  if options.len() == 1 {
    return options[0];
  }
  let displayer = |player: &&Option<PlayerId>| match player {
    Some(player) => game.get_player(*player).name.clone(),
    None => String::from("Aucun"),
  };

  interface.clear_terminal();
  // TODO: check the leader's code to validate
  interface.play_alarm("Merci de faire venir le représentant du personnel!");
  println!("");
  println!("Un des membres d'équipage suivant doit être éliminé:");
  return *interface.user_select_from_with_custom_display(options.iter(), displayer);
}

pub fn run_mutants_phase(game: &mut dyn Game) {
  let current_date = game.get_date(); // do better

    // Notify the mutants of who the other mutants are
    // The newly converted mutant will only get that information at the next night
    let mutants_names = game.get_players().iter()
        .filter_map(|player| if player.infected { Some(player.name.clone())} else { None })
        .collect::<Vec<String>>().join(" ");
    game.limited_broadcast(Message {
        date: current_date,
        source: String::from("Overmind"),
        content: String::from(format!("Lors du dernier crépuscule, les mutant·e·s étaient: [{mutants_names}]")),
    }, & |player: &&mut &mut Player| player.infected);

    for player in game.get_mut_players().iter_mut().filter(|player| player.infected) {
      player.spy_info.woke_up = true;
    }

    // Mutate one player
    let mutate_results = compute_votes_winner(
        game.get_players().iter().filter(|player| player.infected),
        ActionType::Infect);
    if let Some((player_id, _)) = mutate_results {
        let mutatee_name = &game.get_player(player_id).name;
        game.limited_broadcast(Message { // Notify mutants of who was infected
            date: current_date,
            source: String::from("Overmind"),
            content: String::from(format!("Nos spores ont été envoyées dans la cabine de {mutatee_name}, iel devrait bientôt nous rejoindre...")),
        }, & |player: &&mut &mut Player| player.infected);
        let mutate_winner = game.get_mut_player(player_id);
        if mutate_winner.infected == false {
          if mutate_winner.resilient == false {
            mutate_winner.infected = true;
            mutate_winner.spy_info.was_infected = true;
            mutate_winner.send_message(Message { // Notify the new mutant that he was infected
                date: current_date,
                source: String::from("Overmind"),
                content: String::from(format!("Bienvenue {}, nous sommes heureuxe de vous compter parmis nous.", mutate_winner.name)),
            })
          } else {
            mutate_winner.send_message(Message { // Notify the player that he resisted infection
                date: current_date,
                source: String::from("Outil d'auto diagnostique"),
                content: String::from(format!("Bonne nouvelle {}, les mutants ont essayé de vous infecter, mais votre genome vous a protégé!", mutate_winner.name)),
            })
          }
        }
    }

    // Paralyze one player
    let paralyze_result = compute_votes_winner(
        game.get_players().iter().filter(|player| player.infected),
        ActionType::Paralyze);
    if let Some((player_id, _)) = paralyze_result {
        let paralized_name = &game.get_player(player_id).name;
        game.limited_broadcast(Message { // Notify mutants of who was paralysed
            date: current_date,
            source: String::from("Overmind"),
            content: String::from(format!("Félicitations, cette nuit vous êtes parvenus à paralyser: {paralized_name}")),
        }, & |player: &&mut &mut Player| player.infected);

        let paralyzed_player = game.get_mut_player(player_id);
        paralyzed_player.paralyzed = true;
        paralyzed_player.spy_info.was_paralyzed = true;
        paralyzed_player.messages.push(Message {
            date: current_date,
            source: String::from("Outil d'auto diagnostique"),
            content: String::from("Vous avez été paralysé pendant la nuit, vous n'avez donc pas pu faire d'action spéciale"),
        })
    }
}


pub fn run_physicians_phase(game: &mut dyn Game) {
  let current_date = game.get_date(); // do better

  // Cure one player
  let alive_players = game.get_players();
  let physicians = alive_players
    .iter()
    .filter(|player| player.role == Role::Physician);
  let mut cured_players = Vec::new();
  let mut active_physicians = Vec::new();
  let mut active_physician_names = Vec::new();
  let mut disabled_physicians = Vec::new();

  for physician in physicians {
    if physician.infected || physician.paralyzed {
      disabled_physicians.push(physician.id);
    } else {
      active_physicians.push(physician.id);
    }
  }

  for disabled_physician in disabled_physicians.iter() {
    let disabled_physician = game.get_mut_player(*disabled_physician);
    disabled_physician.send_message(Message {
      date: current_date,
      source: String::from("Outil d'auto diagnostique"),
      content: if disabled_physician.infected {
        String::from("Vous êtes infecté·e, vous n'avez donc pas participé aux soins")
      } else {
        String::from("Vous avez été·e paralysé·e pendant la nuit, vous n'avez donc pu participer aux soins")
      }
    });
  }

  for active_physician in active_physicians.iter() {
    let active_physician = game.get_mut_player(*active_physician);
    active_physician.spy_info.woke_up = true;
    active_physician_names.push(active_physician.name.clone());
    if active_physician.auto_cure_physician {
      if let Some(target) = disabled_physicians.pop() {
        cured_players.push(target);
        continue; // not great, but avoids is_empty() + pop().unwrap()
      }
    }
    if let Some(target) = active_physician.actions.get(&ActionType::Cure) {
      cured_players.push(*target);
    }
  }

  // Cure the players, and warn them
  let mut cured_players_names = Vec::new();
  for cured_player in cured_players {
    cured_players_names.push(game.get_player(cured_player).name.clone());
    if game.get_player(cured_player).role == Role::Patient0 {
      game.get_mut_player(cured_player).send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez soigné par un traitement par irradiation intense cette nuit, mais la mutation est trop avancée chez vous, cela a échoué"),
      });
    } else if !game.get_player(cured_player).infected {
      game.get_mut_player(cured_player).send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez été soigné par un traitement anti-mutation cette nuit, bien qu'il n'y ait eu aucune trace de mutations dans votre corps"),
      });
    } else if game.get_player(cured_player).host {
      game.get_mut_player(cured_player).send_message(Message {
        date: current_date,
        source: String::from("Overmind"),
        content: String::from("L'équipe médicale vous a administré un traitement anti-mutation cette nuit, mais votre génome semble résistant au traitement. Félicitations ;-)"),
      });
    } else { // infected and not host
      game.get_mut_player(cured_player).infected = false;
      game.get_mut_player(cured_player).spy_info.was_cured = true;
      game.get_mut_player(cured_player).send_message(Message {
        date: current_date,
        source: String::from("Équipe médicale"),
        content: String::from("Vous avez été soigné par un traitement par irradiation intense cette nuit, qui vous à débarrassé de toute trace de mutation"),
      });
    }
  }

  // Send messages to the active medical team about who was cured
  let active_physician_names = active_physician_names.join(" ");
  let cured_players_names = cured_players_names.join(" ");
  for active_physician in active_physicians {
    let active_physician = game.get_mut_player(active_physician);
    active_physician.send_message(Message {
      date: current_date,
      source: String::from("Équipe médicale"),
      content: String::from(format!("L'équipe médicale opérationelle de la nuit précédente ({}) est parvenue à soigner: [{}]", active_physician_names, cured_players_names)),
    });
  }
}

pub fn run_it_phase(game: &mut dyn Game) {
  let current_date = game.get_date(); // do better

  // Tell the IT guy how many mutants are in play
  let infected_players = game.get_players().iter().filter(|player| player.infected).count();
  for player in game.get_mut_players() {
    if player.role == Role::ITEngineer && !player.paralyzed {
      player.spy_info.woke_up = true;
      player.send_message(Message {
        date: current_date,
        source: String::from("Système de diagnostique"),
        content: format!("L'analyse quantique de cette nuit a révélé la présence de {infected_players} membres d'équipage infectés à bord."),
      })
    } // See if we want to display something in else
  }
}

pub fn run_psychologist_phase(game: &mut dyn Game) {
  let current_date = game.get_date(); // do better
  let psychologists_ids = game.get_player_ids(&|player| player.role == Role::Psychologist);
  for psychologists_id in psychologists_ids {
    if !game.get_player(psychologists_id).paralyzed {
      game.get_mut_player(psychologists_id).spy_info.woke_up = true;
      if let Some(analyzed_id) = game.get_player(psychologists_id).get_target(&ActionType::Psychoanalyze).copied() {
        game.get_mut_player(analyzed_id).spy_info.was_psychoanalyzed = true;
        let name = game.get_player(analyzed_id).name.clone();
        if game.get_player(analyzed_id).infected {
          game.get_mut_player(psychologists_id).send_message(Message {
            date: current_date,
            source: String::from("Freud GPT"),
            content: format!("D'après l'analyse, il semblerait que le comportement déviant de {} ne découle pas d'un trauma d'enfance, mais d'un changement récent. C'est un·e mutant·e!", name),
          });
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

pub fn run_geneticist_phase(game: &mut dyn Game) {
  let current_date = game.get_date(); // do better
  for geneticist_id in game.get_player_ids(&|player| player.role == Role::Geneticist) {
    let geneticist = game.get_player(geneticist_id);
    if geneticist.paralyzed {
      game.get_mut_player(geneticist_id).send_message(Message {
        date: current_date,
        source: String::from("Outil d'auto diagnostique"),
        content: String::from("Vous avez été paralysé·e pendant la nuit, vous n'avez donc pu étudier le genome de vos camarades"),
      });
    } else {
      if let Some(target) = geneticist.get_target(&ActionType::Genomyze).copied() {
        game.get_mut_player(geneticist_id).spy_info.woke_up = true;
        let target_name = game.get_player(target).name.clone();
        let host = game.get_player(target).host;
        let resilient = game.get_player(target).resilient;
        if host {
          game.get_mut_player(geneticist_id).send_message(Message {
            date: current_date,
            source: String::from("GenoTech v0.17"),
            content: format!("Votre analyse du génome de {target_name} révèle qu'il est particulièrement sensible à l'infection. {}",
            Color::FgGreen.color("S'il venait à muter, il ne pourrait être soigné")),
          });
        } else if resilient {
          game.get_mut_player(geneticist_id).send_message(Message {
            date: current_date,
            source: String::from("GenoTech v0.17"),
            content: format!("Votre analyse du génome de {target_name} révèle qu'il est résistant à l'infection. {}",
              Color::FgGreen.color("Il ne deviendra jamais un mutant")),
          });
        } else {
          game.get_mut_player(geneticist_id).send_message(Message {
            date: current_date,
            source: String::from("GenoTech v0.17"),
            content: format!("Votre analyse du génome de {target_name} révèle qu'il est d'une banalité affligeante. Réponse standard à la mutation"),
          });
        }
      }
    }
  }
}

pub fn run_spy_phase(game: &mut dyn Game) {
  let current_date = game.get_date(); // do better
  for spy_id in game.get_player_ids(&|player| player.role == Role::Spy) {
    let spy = game.get_player(spy_id);
    if spy.paralyzed {
      game.get_mut_player(spy_id).send_message(Message {
        date: current_date,
        source: String::from("Outil d'auto diagnostique"),
        content: String::from("Vous avez été paralysé·e pendant la nuit, vous n'avez donc pu espioner vos camarades"),
      });
    } else {
      if let Some(target) = spy.get_target(&ActionType::Spy).copied() {
        let target_name = game.get_player(target).name.clone();
        let spy_info = game.get_player(target).spy_info.clone();
        if spy_info.woke_up {
          game.get_mut_player(spy_id).send_message(Message {
            date: current_date,
            source: String::from("Stalker IV"),
            content: format!("Durant votre surveillance, vous avez vu {target_name} se reveiller et sortir de son dortoir"),
          });
        }
        if spy_info.was_infected {
          game.get_mut_player(spy_id).send_message(Message {
            date: current_date,
            source: String::from("Stalker IV"),
            content: format!("Durant votre surveillance, vous avez vu {target_name} se transformer en mutant·e"),
          });
        }
        if spy_info.was_paralyzed {
          game.get_mut_player(spy_id).send_message(Message {
            date: current_date,
            source: String::from("Stalker IV"),
            content: format!("Durant votre surveillance, vous avez vu {target_name} être paralysé·e"),
          });
        }
        if spy_info.was_cured {
          game.get_mut_player(spy_id).send_message(Message {
            date: current_date,
            source: String::from("Stalker IV"),
            content: format!("Durant votre surveillance, vous avez vu {target_name} guérir de sa mutation"),
          });
        }
        if spy_info.was_psychoanalyzed {
          game.get_mut_player(spy_id).send_message(Message {
            date: current_date,
            source: String::from("Stalker IV"),
            content: format!("Durant votre surveillance, vous avez vu {target_name} être analysé·e par le psychologue"),
          });
        }
      }
    }
  }
}
