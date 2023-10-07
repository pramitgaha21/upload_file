use ic_stable_structures::memory_manager::{MemoryId, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use crate::asset_handler::Asset;
use crate::chunk_handler::Chunk;
use crate::state::MEMORY_MANAGER;

// A memory for upgrades, where data from the heap can be serialized/deserialized.
const UPGRADES: MemoryId = MemoryId::new(0);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

pub fn get_upgrades_memory() -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(UPGRADES))
}

pub fn get_chunk_stable_memory() -> StableBTreeMap<u128, Chunk, Memory>{
    StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
    )
}

pub fn get_asset_stable_memory() -> StableBTreeMap<u128, Asset, Memory>{
    StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
    )
}