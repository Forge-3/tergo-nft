pub mod id_store;
pub mod logs;
use candid::Principal;
use crate::id_store::*;

ic_cdk::export_candid!();