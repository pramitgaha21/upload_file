use std::cell::RefCell;

use candid::{Principal, Encode};
use ic_cdk::api::management_canister::{provisional::CanisterSettings, main::CanisterInstallMode};
use ic_cdk_macros::{query, update};

pub type StorageList = Vec<Principal>;

pub const STORAGE_WASM: &[u8] = &[1, 2];

pub const IN_PROD: bool = false;

thread_local! {
    pub static STORAGE: RefCell<StorageList> = RefCell::new(vec![]);
}

async fn new_canister(flag: bool) -> Principal{
    let arg = ic_cdk::api::management_canister::main::CreateCanisterArgument{
        settings: Some(CanisterSettings{
            compute_allocation: None,
            controllers: Some(vec![ic_cdk::id()]),
            memory_allocation: None,
            freezing_threshold: None,
        })
    };
    let init_arg = Encode!(&flag).unwrap();
    let canister_id = ic_cdk::api::management_canister::main::create_canister(arg, 4_000_000_000_000).await.unwrap().0.canister_id;
    let arg = ic_cdk::api::management_canister::main::InstallCodeArgument{
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: STORAGE_WASM.to_vec(),
        arg: init_arg
    };
    ic_cdk::api::management_canister::main::install_code(arg).await.unwrap();
    canister_id
}

#[update]
pub async fn query_storage_canister() -> Principal{
    let last_canister = STORAGE.with(|s|{
        let canisters = s.borrow();
        canisters[canisters.len() - 1]
    });
    let res: (bool,) = ic_cdk::call(last_canister, "is_full", ()).await.unwrap();
    match res{
        (false,) => last_canister,
        (true,) => {
            let new_canister = new_canister(IN_PROD).await;
            STORAGE.with(|s| s.borrow_mut().push(new_canister.clone()));
            new_canister
        }
    }
}

#[query]
pub fn storage_list() -> Vec<Principal>{
    STORAGE.with(|s| s.borrow().clone())
}