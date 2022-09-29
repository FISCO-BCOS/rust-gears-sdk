#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_imports,
    unused_results,
    unused_variables
)]
use crate::console::cli_common::Cli;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcos2client::Bcos2Client;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcossdkquery::json_hextoint;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contractabi::ContractABI;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contracthistory::ContractHistory;
use fisco_bcos_rust_gears_sdk::bcossdkutil::fileutils;
use fisco_bcos_rust_gears_sdk::bcossdkutil::kisserror::KissError;
use serde_json::{json, Value as JsonValue};
use std::thread;
use std::time::Duration;

//---------------------------------------------------------
pub fn demo(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str()).unwrap();
    let res = bcossdk.queryGroupStatus(1)?;
    println!("querygroupstatus {:?}", res);
    Ok(())
}
