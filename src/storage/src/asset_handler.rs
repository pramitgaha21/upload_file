use candid::{CandidType, Deserialize, Principal, Decode, Encode};
use ic_cdk_macros::{query, update};
use ic_stable_structures::{Storable, storable::Bound};
use std::{collections::HashMap, time::Duration};
use serde::Serialize;
use crate::{state::STATE, utils::{update_storage, url_generator}, chunk_handler::delete_expired_chunks};

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub asset_id: u128,
    pub file_name: String,
    pub file_type: String,
    pub chunks: Vec<Vec<u8>>,
    pub url: String,
    pub owned_by: Principal,
    pub uploaded_at: u64,
}

impl Storable for Asset{
    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(&bytes, Self).unwrap()
    }

    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(&self).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize)]
pub struct AssetQuery {
    pub asset_id: u128,
    pub file_name: String,
    pub file_type: String,
    pub url: String,
    pub owned_by: Principal,
    pub uploaded_at: u64,
}

impl From<&Asset> for AssetQuery {
    fn from(value: &Asset) -> Self {
        Self {
            asset_id: value.asset_id,
            file_name: value.file_name.clone(),
            file_type: value.file_name.clone(),
            url: value.url.clone(),
            owned_by: value.owned_by,
            uploaded_at: value.uploaded_at,
        }
    }
}

#[derive(CandidType, Deserialize)]
pub struct CommitBatchArgs {
    pub chunk_ids: Vec<u128>,
    pub checksum: u32,
    pub file_name: String,
    pub file_type: String,
}

#[update]
pub fn commit_batch(args: CommitBatchArgs) -> u128 {
    if args.chunk_ids.len() == 0{
        ic_cdk::trap("No ids Provided")
    }
    let caller = ic_cdk::caller();
    STATE.with(|s|{
        let mut state = s.borrow_mut();
        let mut chunks_to_commit = vec![];
        let mut chunks_not_found = vec![];
        let mut chunks_not_owned = vec![];
        args.chunk_ids
            .iter()
            .for_each(|id| match state.chunk_list.get(id) {
                None => {
                    ic_cdk::print("No chunk found");
                    chunks_not_found.push(id.clone())
                },
                Some(chunk) if chunk.owned_by != caller => {
                    ic_cdk::print("Chunk not owned");
                    chunks_not_owned.push(id.clone())
                },
                Some(chunk) => {
                    ic_cdk::print("chunk found");
                    chunks_to_commit.push((id.clone(), chunk.order))
                },
            });
        if chunks_to_commit.len() == 0{
            ic_cdk::trap("No chunks found")
        }
        if chunks_not_found.len() > 0 {
            let error_msg = format!("Chunks not found: {:?}", chunks_not_found);
            ic_cdk::trap(&error_msg)
        }
        if chunks_not_owned.len() > 0 {
            let error_msg = format!("Chunks not owned: {:?}", chunks_not_owned);
            ic_cdk::trap(&error_msg)
        }
        let mut checksum: u32 = 0;
        let mut content = Vec::new();
        let mut chunk_size = 0;
        chunks_to_commit.sort_by_key(|chunks| chunks.1);
        chunks_to_commit.iter().for_each(|(id, _)| {
            let chunk = state.chunk_list.remove(id).unwrap();
            checksum = (checksum + chunk.checksum) % 400_000_000;
            chunk_size += 1;
            content.push(chunk.content);
        });
        if args.checksum != checksum {
            let error_msg = format!("Checksum mismatch: {} != {}", args.checksum, checksum);
            ic_cdk::trap(&error_msg)
        }
        // if content.len() as u32 > <Asset as BoundedStorable>::MAX_SIZE {
        //     ic_cdk::trap("Exceeds allow file limit size")
        // }
        let id = state.get_asset_id();
        let url = url_generator(&id);
        let asset = Asset { asset_id: id, file_name: args.file_name, file_type: args.file_type, chunks: content, url, owned_by: caller, uploaded_at: ic_cdk::api::time() };
        state.asset_list.insert(id, asset).expect("failed to insert");
        id
    })
}

#[update]
pub fn delete_asset(asset_id: u128) -> bool {
    let caller = ic_cdk::caller();
    if caller == Principal::anonymous() {
        ic_cdk::trap("Anonymous Caller")
    }
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        match state.asset_list.get(&asset_id) {
            None => ic_cdk::trap("Invalid Asset Id"),
            Some(asset) if asset.owned_by != caller => ic_cdk::trap("Unauthorized Owner"),
            Some(_) => (),
        }
        state.asset_list.remove(&asset_id);
        ic_cdk_timers::set_timer(Duration::from_secs(0), || ic_cdk::spawn(update_storage()));
        true
    })
}

#[query]
pub fn query_asset(id: u128) -> Option<AssetQuery> {
    STATE.with(|s| {
        let s = s.borrow();
        match s.asset_list.get(&id) {
            None => None,
            Some(ref asset) => Some(AssetQuery::from(asset)),
        }
    })
}

#[query]
pub fn assets_of(principal: Principal) -> HashMap<u128, AssetQuery> {
    let mut assets = HashMap::new();
    STATE.with(|s| {
        let s = s.borrow();
        s.asset_list.iter().for_each(|(id, ref asset)| {
            if asset.owned_by == principal {
                assets.insert(id.clone(), AssetQuery::from(asset));
            }
        });
        assets
    })
}

#[query]
pub fn asset_list() -> HashMap<u128, AssetQuery> {
    STATE.with(|s| {
        let s = s.borrow();
        s.asset_list
            .iter()
            .map(|(id, ref asset)| (id.clone(), AssetQuery::from(asset)))
            .collect()
    })
}
