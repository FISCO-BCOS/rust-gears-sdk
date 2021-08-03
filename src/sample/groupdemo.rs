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
use std::time::Duration;
use crate::bcossdk::bcossdk::BcosSDK;
use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::kisserror::KissError;
use crate::bcossdk::{bcossdkquery, fileutils};
use std::thread;
use serde_json::{json, Value as JsonValue};
use crate::bcossdk::contracthistory::ContractHistory;
use crate::bcossdk::bcossdkquery::json_hextoint;
use crate::bcossdk::cli_common::Cli;
//---------------------------------------------------------
pub fn demo(cli:&Cli)->Result<(),KissError>
{
    let mut bcossdk = BcosSDK::new_from_config(cli.default_configfile().as_str()).unwrap();
    let res = bcossdk.queryGroupStatus(1)?;
    println!("querygroupstatus {:?}",res);
    Ok(())
}