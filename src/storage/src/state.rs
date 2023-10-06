use crate::{
    asset_handler::Asset,
    chunk_handler::Chunk,
    memory::{get_asset_stable_memory, get_chunk_stable_memory, Memory},
};
use candid::{Deserialize, Nat};
use ic_stable_structures::StableBTreeMap;
use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};
use std::cell::RefCell;

#[derive(serde::Serialize, Deserialize)]
pub struct State {
    pub in_prod: bool,
    pub chunk_count: u128,
    pub asset_count: u128,
    pub used_storage: Nat,
    #[serde(skip, default = "get_chunk_stable_memory")]
    pub chunk_list: StableBTreeMap<u128, Chunk, Memory>,
    #[serde(skip, default = "get_asset_stable_memory")]
    pub asset_list: StableBTreeMap<u128, Asset, Memory>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            in_prod: false,
            chunk_count: 0,
            asset_count: 0,
            used_storage: Nat::from(0),
            chunk_list: get_chunk_stable_memory(),
            asset_list: get_asset_stable_memory(),
        }
    }
}

impl State {
    pub fn get_chunk_id(&mut self) -> u128 {
        let id = self.chunk_count.clone();
        self.chunk_count += 1;
        id
    }

    pub fn get_asset_id(&mut self) -> u128 {
        let id = self.asset_count.clone();
        self.asset_count += 1;
        id
    }
}

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    pub static STATE: RefCell<State> = RefCell::default();
}

pub(crate) fn query_in_prod() -> bool {
    STATE.with(|state| state.borrow().in_prod)
}

// if the returned value is of lenth 0, then every ids are valid else
// returns a list of ids that aren't present. use the returned data for better error handling
pub fn chunk_ids_validity_check(ids: &[u128]) -> Vec<u128> {
    STATE.with(|state| {
        let state = state.borrow();
        let mut invalid_ids = vec![];
        ids.iter().for_each(|id| {
            if !state.chunk_list.contains_key(id) {
                invalid_ids.push(id.clone());
            }
        });
        invalid_ids
    })
}
