pub mod state;
pub mod types;
pub mod guards;
pub mod cycles;
pub mod init;
pub mod query_method;
pub mod update_method;

use crate::cycles::WalletReceiveResult;
use crate::types::*;
use candid::Principal;
use icrc_ledger_types::icrc3::blocks::GetBlocksRequest;

ic_cdk::export_candid!();