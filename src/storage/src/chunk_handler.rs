use std::time::Duration;

use candid::{CandidType, Deserialize, Principal, Encode, Decode};
use ic_cdk_macros::{query, update};
use ic_stable_structures::{Storable, storable::Bound};

use crate::{state::{chunk_ids_validity_check, STATE}, utils::update_storage};
use serde::Serialize;

// In seconds
const EXPIRY_LIMIT: u64 = 10 * 60 * 60 * 1000_000;

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub chunk_id: u128, // 16 bytes
    pub order: u32, // 4 bytes
    pub content: Vec<u8>, // 2 * 1024 * 1024 * 8 bytes
    pub owned_by: Principal, // 30 bytes
    pub uploaded_at: u64, // 8 bytes
    // pub checksum: u32, // 4 bytes
}

impl Storable for Chunk{
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    const BOUND: Bound = Bound::Bounded { max_size: 16 + 4 + 2 * 1024 * 1024 * 8 + 30 + 8, is_fixed_size: false };
}

#[derive(CandidType, Deserialize)]
pub struct ChunkArgs {
    pub order: u32,
    pub content: Vec<u8>,
}

impl From<(u128, ChunkArgs, Principal)> for Chunk {
    fn from((chunk_id, arg, owned_by): (u128, ChunkArgs, Principal)) -> Self {
        // let checksum = crc32fast::hash(&arg.content);
        // ic_cdk::println!("{}", checksum);
        Self {
            chunk_id,
            order: arg.order,
            content: arg.content,
            owned_by,
            uploaded_at: ic_cdk::api::time(),
            // checksum,
        }
    }
}

#[update]
pub fn upload_chunk(arg: ChunkArgs) -> u128 {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        ic_cdk::trap("Anonymous Caller")
    }
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let chunk_id = state.get_chunk_id();
        let chunk = Chunk::from((chunk_id.clone(), arg, caller));
        state.chunk_list.insert(chunk_id.clone(), chunk);
        ic_cdk_timers::set_timer(Duration::from_secs(0), || ic_cdk::spawn(update_storage()));
        chunk_id
    })
}

#[update]
pub fn delete_expired_chunks() {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let current_time = ic_cdk::api::time();
        let mut expired_ids: Vec<u128> = vec![];
        state.chunk_list.iter().for_each(|(id, chunk)| {
            let allowed_time = chunk.uploaded_at + EXPIRY_LIMIT;
            if current_time > allowed_time{
                expired_ids.push(id.clone());
            }
        });
        expired_ids.iter().for_each(|id| {
            state.chunk_list.remove(id);
        });
        ic_cdk_timers::set_timer(Duration::from_secs(0), || ic_cdk::spawn(update_storage()));
    })
}

#[query]
pub fn chunk_ids_check(ids: Vec<u128>) -> bool {
    chunk_ids_validity_check(&ids)
}