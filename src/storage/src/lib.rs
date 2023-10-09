use ic_cdk_macros::export_candid;
use std::collections::HashMap;
use candid::Principal;

pub mod init;
pub mod memory;
pub mod asset_handler;
pub mod canister_method;
pub mod chunk_handler;
pub mod http_handler;
pub mod state;
pub mod utils;
// pub mod candid_file_generator;

pub use asset_handler::*;
pub use http_handler::*;
pub use chunk_handler::*;

export_candid!();