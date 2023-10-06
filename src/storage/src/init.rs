use ic_cdk_macros::init;

use crate::state::STATE;

#[init]
pub fn init(in_prod: bool) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.in_prod = in_prod;
    })
}
