 // Import Error from core::fmt if needed
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage, StdResult};
use cw_storage_plus::{Item, Map};

pub static VOTE_KEY_PREFIX: &[u8] = b"vote";

// Define storage keys
pub const VOTE_STORE: Map<&Addr, Vote> = Map::new("votes");
pub const CONFIG: Item<State> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct Vote {
    pub voter: Addr,
    pub candidate_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

// Function to store a vote
pub fn store_vote(storage: &mut dyn Storage, voter_address: &Addr, candidate_id: u32) -> StdResult<()> {
    let vote = Vote {
        voter: voter_address.clone(),
        candidate_id,
    };
    VOTE_STORE.save(storage, voter_address, &vote)?;
    Ok(())
}

// Function to read all votes
pub fn read_votes(storage: &dyn Storage) -> StdResult<Vec<Vote>> {
    let votes: StdResult<Vec<Vote>> = VOTE_STORE.range(storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| item.map(|(_, vote)| vote))
        .collect();
    votes
}
