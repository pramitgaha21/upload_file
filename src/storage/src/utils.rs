use ic_cdk::api::management_canister::provisional::CanisterIdRecord;

use crate::state::STATE;

pub fn url_generator(in_prod: &bool, asset_id: &u128) -> String {
    let canister_id = ic_cdk::id();
    match in_prod {
        true => format!("https://{canister_id}.raw.ic0.app/asset/{asset_id}"),
        false => format!("http://{canister_id}.localhost:8080/asset/{asset_id}"),
    }
}

pub fn asset_id_extractor(url: &str) -> u128 {
    let url_split_by_path = url.split('/').collect::<Vec<&str>>();
    let last_elem = url_split_by_path[url_split_by_path.len() - 1];
    let first_elem: Vec<&str> = last_elem.split('?').collect();
    let element = first_elem[0].trim().parse::<u128>().unwrap();
    element
}

pub async fn update_storage() {
    let id_record = CanisterIdRecord{
        canister_id: ic_cdk::id(),
    };
    let status = ic_cdk::api::management_canister::main::canister_status(id_record).await
        .unwrap().0;
    STATE.with(|s|{
        let mut s = s.borrow_mut();
        s.used_storage = status.memory_size;
    })
}
