extern crate cosmwasm_std;
extern crate schemars;
extern crate serde;


use cosmwasm_std::{
    to_json_binary, from_json, Binary, CanonicalAddr, Env, Response, StdError, StdResult, Storage,
    Deps, DepsMut, MessageInfo,  Order,
};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};



// Define PollStruct and ContestantStruct as before...

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
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
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ContestantStruct {
    pub id: u64,
    pub image: String,
    pub name: String,
    pub voter: CanonicalAddr,
    pub votes: u64,
    pub voters: Vec<CanonicalAddr>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
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
    info: MessageInfo,
    msg: HandleMsg,
) -> StdResult<Response> {
    match msg {
        HandleMsg::CreatePoll { image, title, description, starts_at, ends_at } => {
            create_poll(deps, env, info, image, title, description, starts_at, ends_at)
        },
        HandleMsg::UpdatePoll { id, image, title, description, starts_at, ends_at } => {
            update_poll(deps, env, info, id, image, title, description, starts_at, ends_at)
        },
        HandleMsg::DeletePoll { id } => delete_poll(deps, env, info, id),
        HandleMsg::Contest { poll_id, name, avatar } => contest(deps, env, info, poll_id, name, avatar),
        HandleMsg::Vote { poll_id, contestant_id } => vote(deps, env, info, poll_id, contestant_id),
    }
}

fn create_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    image: String,
    title: String,
    description: String,
    starts_at: u64,
    ends_at: u64,
) -> StdResult<Response> {
    // Fetch sender's address
    let sender = deps.api.addr_canonicalize(info.sender.as_str())?;

    // Generate a unique poll ID
    let poll_id = generate_unique_poll_id(deps.storage)?;

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
        question: String::new(),
        options: Vec::new(),
    };

    // Store the poll in storage
    set_poll(deps.storage, &poll_id.to_be_bytes(), &poll)?;

    Ok(Response::default())
}

const POLL_COUNT_KEY: &str = "poll_count";

fn generate_unique_poll_id(storage: &mut dyn Storage) -> StdResult<u64> {
    let poll_count_store = Map::new( POLL_COUNT_KEY); // Corrected: Use `Map::new(storage, key)` to initialize `poll_count_store`
    let count = poll_count_store.may_load(storage, &[] as &[u8])?.unwrap_or_default(); // Corrected: Use `may_load()` directly without passing `storage` and empty slice
    let new_count = count + 1;
    poll_count_store.save(storage, &[], &new_count)?; // Corrected: Use `save()` directly with `&new_count`
    Ok(new_count)
}




fn set_poll(storage: &mut dyn Storage, id: &[u8], poll: &PollStruct) -> StdResult<()> {
    let poll_bin = to_json_binary(poll)?;
    storage.set(id, &poll_bin);
    Ok(())
}

fn update_poll(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    id: u64,
    image: String,
    title: String,
    description: String,
    starts_at: u64,
    ends_at: u64,
) -> StdResult<Response> {
    let poll_key = id.to_be_bytes();
    let mut poll = load_poll(deps.as_ref(), id)?;

    poll.image = image;
    poll.title = title;
    poll.description = description;
    poll.starts_at = starts_at;
    poll.ends_at = ends_at;

    set_poll(deps.storage, &poll_key, &poll)?;

    Ok(Response::default())
}

fn delete_poll(deps: DepsMut, _env: Env, _info: MessageInfo, id: u64) -> StdResult<Response> {
    let poll_key = id.to_be_bytes();
    let mut poll = load_poll(deps.as_ref(), id)?;

    poll.deleted = true;

    set_poll(deps.storage, &poll_key, &poll)?;

    Ok(Response::default())
}

fn contest(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    poll_id: u64,
    name: String,
    avatar: String,
) -> StdResult<Response> {
    let poll_key = poll_id.to_be_bytes();
    let mut poll = load_poll(deps.as_ref(), poll_id)?;

    let contestant_id = poll.contestants + 1;
    poll.contestants = contestant_id;

    let contestant = ContestantStruct {
        id: contestant_id,
        image: avatar.clone(),
        name: name.clone(),
        voter: deps.api.addr_canonicalize(info.sender.as_str())?,
        votes: 0,
        voters: vec![],
    };

    set_poll(deps.storage, &poll_key, &poll)?;
    set_contestant(deps.storage, poll_id, &contestant)?;

    Ok(Response::default())
}

fn vote(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    poll_id: u64,
    contestant_id: u64,
) -> StdResult<Response> {
    let poll_key = poll_id.to_be_bytes();
    let mut poll = load_poll(deps.as_ref(), poll_id)?;
    let mut contestant = load_contestant(deps.storage, poll_id, contestant_id)?;

    let voter = deps.api.addr_canonicalize(info.sender.as_str())?;

    if poll.voters.contains(&voter) {
        return Err(StdError::generic_err("You have already voted in this poll"));
    }

    poll.votes += 1;
    poll.voters.push(voter.clone());

    contestant.votes += 1;
    contestant.voters.push(voter);

    set_poll(deps.storage, &poll_key, &poll)?;
    set_contestant(deps.storage, poll_id, &contestant)?;

    Ok(Response::default())
}

fn set_contestant(storage: &mut dyn Storage, poll_id: u64, contestant: &ContestantStruct) -> StdResult<()> {
    let key = format!("contestant_{}_{}", poll_id, contestant.id).into_bytes();
    let contestant_bin = to_json_binary(contestant)?;
    storage.set(&key, &contestant_bin);
    Ok(())
}

pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPolls {} => get_polls(deps),
        QueryMsg::GetPoll { id } => get_poll(deps, id),
        QueryMsg::GetContestants { poll_id } => get_contestants(deps, poll_id),
        QueryMsg::GetContestant { poll_id, contestant_id } => get_contestant(deps, poll_id, contestant_id),
    }
}

fn get_polls(deps: Deps) -> StdResult<Binary> {
    let polls: Vec<PollStruct> = load_all_polls(deps)?;
    to_json_binary(&polls)
}

fn load_all_polls(deps: Deps) -> StdResult<Vec<PollStruct>> {
    let prefix = b"poll";
    let polls: StdResult<Vec<PollStruct>> = deps.storage.range(Some(prefix), None, Order::Ascending)
        .map(|item| {
            let (_, value) = item;
            let poll: PollStruct = from_json(&value)?;
            Ok(poll)
        })
        .collect();
    polls
}

fn get_poll(deps: Deps, id: u64) -> StdResult<Binary> {
    let poll = load_poll(deps, id)?;
    to_json_binary(&poll)
}

fn load_poll(deps: Deps, id: u64) -> StdResult<PollStruct> {
    let poll_key = id.to_be_bytes();
    let poll_bin = deps.storage.get(&poll_key).ok_or_else(|| StdError::generic_err("Poll not found"))?;
    let poll: PollStruct = from_json(&poll_bin)?;
    Ok(poll)
}

fn get_contestants(deps: Deps, poll_id: u64) -> StdResult<Binary> {
    let contestants: Vec<ContestantStruct> = load_contestants(deps.storage, poll_id)?;
    to_json_binary(&contestants)
}

fn load_contestants(storage: &dyn Storage, poll_id: u64) -> StdResult<Vec<ContestantStruct>> {
    let prefix = format!("contestants_{}", poll_id).into_bytes();
    let contestants: StdResult<Vec<ContestantStruct>> = storage.range(Some(&prefix), None, Order::Ascending)
        .map(|item| {
            let (_, value) = item;
            let contestant: ContestantStruct = from_json(&value)?;
            Ok(contestant)
        })
        .collect();
    contestants
}

fn get_contestant(deps: Deps, poll_id: u64, contestant_id: u64) -> StdResult<Binary> {
    let contestant = load_contestant(deps.storage, poll_id, contestant_id)?;
    to_json_binary(&contestant)
}

fn load_contestant(storage: &dyn Storage, poll_id: u64, contestant_id: u64) -> StdResult<ContestantStruct> {
    let key = format!("contestant_{}_{}", poll_id, contestant_id).into_bytes();
    let contestant_bin = storage.get(&key).ok_or_else(|| StdError::generic_err("Contestant not found"))?;
    let contestant: ContestantStruct = from_json(&contestant_bin)?;
    Ok(contestant)
}
