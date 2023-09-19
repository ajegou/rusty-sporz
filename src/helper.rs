use std::collections::HashMap;

use crate::{action::ActionType, player::{PlayerId, Player}};


// Returns the ID of the player who received the most votes for the given action among the voters, along the number of votes
// If several players received the highest number of votes, one is selected at random
pub fn compute_votes_winner <'a, T> (voters: T, action: ActionType) -> Option<(PlayerId, usize)>
where T: IntoIterator<Item = &'a&'a Player>,
{
    let results = compute_votes_results(voters, action);
    let mut winner: Option<PlayerId> = None;
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

pub fn compute_votes_results <'a, T> (voters: T, action: ActionType) -> HashMap<PlayerId, usize>
where T: IntoIterator<Item = &'a&'a Player>,
{
    let results = voters.into_iter().filter_map(|mutant| mutant.actions.get(&action))
    .fold(HashMap::new(), |mut acc, target| {
        *acc.entry(*target).or_insert(0) += 1;
        acc
    });
    return results;
}
