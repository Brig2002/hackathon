extern crate cosmwasm_std;
extern crate schemars;
extern crate serde;
extern crate cosmwasm_storage;

use cosmwasm_std::{
    to_binary, from_slice, Binary, CanonicalAddr, Env, Response, StdError, StdResult, Storage,
    Deps, DepsMut, MessageInfo
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
 


#[derive(Debug, Serialize)]




#[derive(Clone,  PartialEq,  )]
pub struct PollStruct {
    pub id: u64,
    pub image: String,
    pub title: String,
    pub description: String,
    pub votes: u64,
    pub contestants: u64,
    pub deleted: bool,
    pub director: CanonicalAddr,
    pub starts_at: u64,
    pub ends_at: u64,
    pub timestamp: u64,
    pub voters: Vec<CanonicalAddr>,
    pub avatars: Vec<String>,
    question: String,
    options: Vec<String>,
}

#[derive( Clone, Debug, PartialEq )]
pub struct ContestantStruct {
    pub id: u64,
    pub image: String,
    pub name: String,
    pub voter: CanonicalAddr,
    pub votes: u64,
    pub voters: Vec<CanonicalAddr>,
}

#[derive( Clone, Debug, PartialEq)]
pub struct InitMsg {}

#[derive( Clone, Debug, PartialEq)]
pub enum HandleMsg {
    CreatePoll {
        image: String,
        title: String,
        description: String,
        starts_at: u64,
        ends_at: u64,
    },
    UpdatePoll {
        id: u64,
        image: String,
        title: String,
        description: String,
        starts_at: u64,
        ends_at: u64,
    },
    DeletePoll { id: u64 },
    Contest {
        poll_id: u64,
        name: String,
        avatar: String,
    },
    Vote { poll_id: u64, contestant_id: u64 },
}

#[derive( Clone, Debug, PartialEq)]
pub enum QueryMsg {
    GetPolls {},
    GetPoll { id: u64 },
    GetContestants { poll_id: u64 },
    GetContestant { poll_id: u64, contestant_id: u64 },
}

pub fn init(_deps: DepsMut, _env: Env, _msg: InitMsg) -> StdResult<Response> {
    Ok(Response::default())
}

pub fn handle(
    deps: DepsMut, 
    env: Env, 
    msg: HandleMsg,
) -> StdResult<Response> {
    match msg {
        HandleMsg::CreatePoll { image, title, description, starts_at, ends_at } => {
            create_poll(deps, env, image, title, description, starts_at, ends_at)
        },
        HandleMsg::UpdatePoll { id, image, title, description, starts_at, ends_at } => {
            update_poll(deps, env, id, image, title, description, starts_at, ends_at)
        },
        HandleMsg::DeletePoll { id } => delete_poll(deps, env, id),
        HandleMsg::Contest { poll_id, name, avatar } => contest(deps, env, poll_id, name, avatar),
        HandleMsg::Vote { poll_id, contestant_id } => vote(deps, env, poll_id, contestant_id),
    }
}

fn create_poll(
    deps: DepsMut,
    env: Env,
    image: String,
    title: String,
    description: String,
    starts_at: u64,
    ends_at: u64,
) -> StdResult<Response> {
    // Fetch sender's address
    let sender = &deps.api.canonical_address(&env.message.sender)?;

    // Generate a unique poll ID
    let poll_id = generate_unique_poll_id( deps.storage)?;

    // Create a new poll
    let poll = PollStruct {
        id: poll_id,
        image,
        title,
        description,
        votes: 0,
        contestants: 0,
        deleted: false,
        director: sender.clone(),
        starts_at,
        ends_at,
        timestamp: env.block.time.seconds(),
        voters: vec![],
        avatars: vec![],
        question: String,
        options: Vec<String>,

    };

    // Store the poll in storage
    set_poll( deps.storage, &poll_id.to_be_bytes(), &poll)?;

    Ok(Response::default())
}

fn generate_unique_poll_id(storage: &mut dyn Storage) -> StdResult<u64> {
    let mut poll_count_store = PrefixedStorage::new(storage, b"poll_count");
    let count = poll_count_store.may_load::<u64>(&[])?;
    let new_count = count.map_or(1, |c| c + 1);
    poll_count_store.save(&[], &new_count)?;
    Ok(new_count)
}

fn set_poll(storage: &mut dyn Storage, id: &[u8], poll: &PollStruct) -> StdResult<()> {
    // Convert poll to binary
    let poll_bin = to_binary(poll)?;

    // Store poll in storage
    storage.set(id, &poll_bin);

    Ok(())
}

pub fn  query(
    _deps: Deps,

    _env: Env,
    msg: QueryMsg,
 ) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPolls {} => get_polls(),
        QueryMsg::GetPoll { id } => get_poll(&_deps, id,_env),
        QueryMsg::GetContestants { poll_id } => get_contestants(&mut poll_id ),
        QueryMsg::GetContestant { poll_id, contestant_id } => get_contestant(&mut  poll_id, contestant_id, _env),
    }
}

fn get_contestants(deps: Deps, poll_id: u64) -> StdResult<Binary> {
    // Fetch contestants for a poll from storage
    let contestants: Vec<ContestantStruct> = load_contestants(deps. storage, poll_id)?;
    to_binary(&contestants)
}

fn get_polls(deps: Deps,env: Env) -> StdResult<Binary> {
    // Fetch all polls from storage
    let polls: Vec<PollStruct> = load_all_polls( deps)?;
    to_binary(&polls)
}

fn load_all_polls(deps: Deps) -> StdResult<Vec<PollStruct>> {
    let prefix = b"poll";
    let polls: StdResult<Vec<PollStruct>> = Storage::scan(PrefixedStorage, prefix)
        .map(|item| {
            let (_, value) = item?;
            let poll: PollStruct = from_slice(&value)?;
            Ok(poll)
        })
        .collect();
    polls
}

fn get_poll(_deps: Deps, id: u64) -> StdResult<Binary> {
    // Fetch poll by ID from storage
    let poll = load_poll(&_deps, id)?;
    to_binary(&poll)
}

fn load_poll(deps: &Deps, id: u64) -> StdResult<PollStruct> {
    let poll_key = id.to_be_bytes();
    let poll_bin = Storage::get(deps.storage, &poll_key).ok_or_else(|| StdError::generic_err("Poll not found"))?;
    let poll: PollStruct = from_slice(&poll_bin)?;
    Ok(poll)
}

fn get_contestant(_deps: Deps, poll_id: u64, contestant_id: u64) -> StdResult<Binary> {
    // Fetch contestant by ID for a poll from storage
    let contestant = load_contestant(_deps, poll_id, contestant_id)?;
    to_binary(&contestant)
}

fn load_contestants(storage: &dyn Storage, poll_id: u64) -> StdResult<Vec<ContestantStruct>> {
    let prefix = format!("contestants_{}", poll_id).into_bytes();
    let contestants: StdResult<Vec<ContestantStruct>> = Storage::scan(storage, &prefix[..])
        .map(|item| {
            let (_, value) = item?;
            let contestant: ContestantStruct = from_slice(&value)?;
            Ok(contestant)
        })
        .collect();
    contestants
}

fn load_contestant(storage: &dyn Storage, poll_id: u64, contestant_id: u64) -> StdResult<ContestantStruct> {
    let key = format!("contestant_{}_{}", poll_id, contestant_id).into_bytes();
    let contestant_bin = storage.get(&key).ok_or_else(|| StdError::generic_err("Contestant not found"))?;
    let contestant: ContestantStruct = from_slice(&contestant_bin)?;
    Ok(contestant)
}

fn delete_poll(deps: DepsMut, _env: Env, id: u64) -> StdResult<Response> {
    // Implement delete poll logic here
    Ok(Response::default())
}

fn update_poll(
    deps: DepsMut,
    _env: Env,
    id: u64,
    image: String,
    title: String,
    description: String,
    starts_at: u64,
    ends_at: u64,
) -> StdResult<Response> {
    // Implement update poll logic here
    Ok(Response::default())
}

fn contest(
    deps: DepsMut,
    _env: Env,
    poll_id: u64,
    name: String,
    avatar: String,
) -> StdResult<Response> {
    // Implement contest logic here
    Ok(Response::default())
}

fn vote(
    deps: DepsMut,
    _env: Env,
    poll_id: u64,
    contestant_id: u64,
) -> StdResult<Response> {
    // Implement vote logic here
    Ok(Response::default())
} 




