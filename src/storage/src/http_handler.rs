use crate::{state::STATE, utils::asset_id_extractor};
use candid::{Func, CandidType, Deserialize, define_function};
use ic_cdk_macros::query;

#[derive(CandidType, Deserialize, Clone)]
pub struct HeaderField(pub String, pub String);

#[derive(CandidType, Deserialize, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: Vec<u8>,
    pub streaming_strategy: Option<StreamingStrategy>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CreateStrategyArgs {
    pub asset_id: u128,
    pub chunk_index: u32,
    pub chunk_size: u32,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct StreamingCallbackToken {
    pub asset_id: u128,
    pub chunk_index: u32,
    pub chunk_size: u32,
    pub content_encoding: String,
}

#[derive(CandidType, Deserialize, Clone)]
pub enum StreamingStrategy {
    Callback {
        token: StreamingCallbackToken,
        callback: CallbackFunc,
    },
}

#[derive(CandidType, Deserialize, Clone)]
pub struct StreamingCallbackHttpResponse {
    pub body: Vec<u8>,
    pub token: Option<StreamingCallbackToken>,
}

define_function!(pub CallbackFunc: () -> () query);

#[query]
pub fn http_request(request: HttpRequest) -> HttpResponse {
    let not_found = b"Asset Not Found".to_vec();
    let asset_id = asset_id_extractor(&request.url);
    STATE.with(|state| {
        let state = state.borrow();
        match state.asset_list.get(&asset_id) {
            None => HttpResponse {
                body: not_found,
                status_code: 404,
                headers: vec![],
                streaming_strategy: None,
            },
            Some(asset) => {
                let filename = format!("attachment; filename={}", asset.file_name.clone());
                HttpResponse {
                    body: asset
                        .chunks[0_usize].clone(),
                    status_code: 200,
                    headers: vec![
                        HeaderField("Content-Type".to_string(), asset.file_type.clone()),
                        HeaderField("accept-ranges".to_string(), "bytes".to_string()),
                        HeaderField("Content-Disposition".to_string(), filename),
                        HeaderField(
                            "cache-control".to_string(),
                            "private, max-age=0".to_string(),
                        ),
                    ],
                    streaming_strategy: create_strategy(CreateStrategyArgs {
                        asset_id: asset_id.clone(),
                        chunk_index: 0,
                        chunk_size: asset.chunks.len() as u32,
                    }),
                }
            }
        }
    })
}

fn create_strategy(arg: CreateStrategyArgs) -> Option<StreamingStrategy> {
    match create_token(arg) {
        None => None,
        Some(token) => {
            let id = ic_cdk::id();
            Some(StreamingStrategy::Callback {
                token,
                callback: CallbackFunc::new(id, "http_request_streaming_callback".to_string())
            })
        }
    }
}

fn create_token(arg: CreateStrategyArgs) -> Option<StreamingCallbackToken> {
    let v = arg.chunk_index.clone() + 1;
    if v >= arg.chunk_size {
        return None;
    }
    Some(StreamingCallbackToken {
        asset_id: arg.asset_id,
        chunk_index: arg.chunk_index + 1,
        content_encoding: "gzip".to_string(),
        chunk_size: arg.chunk_size as u32,
    })
}

#[query]
pub fn http_request_streaming_callback(
    token_arg: StreamingCallbackToken,
) -> StreamingCallbackHttpResponse {
    STATE.with(|state| {
        let state = state.borrow();
        match state.asset_list.get(&token_arg.asset_id) {
            None => panic!("asset id not found"),
            Some(asset) => {
                let arg = CreateStrategyArgs {
                    asset_id: token_arg.asset_id.clone(),
                    chunk_index: token_arg.chunk_index,
                    chunk_size: token_arg.chunk_size,
                };
                let token = create_token(arg);
                StreamingCallbackHttpResponse {
                    token,
                    body: asset.chunks[token_arg.chunk_index as usize].clone(),
                }
            }
        }
    })
}