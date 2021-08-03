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
mod sample;
mod bcossdk;
mod console;

use std::{env, thread};
use crate::bcossdk::kisserror::{KissError,KissErrKind};
use std::env::Args;


use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::bcossdk::BcosSDK;
use std::time::Duration;
use crate::bcossdk::bcossdkquery;
use crate::bcossdk::contracthistory::ContractHistory;
use crate::console::{console_account, console_contract};
use crate::console::console_cmds;
use crate::bcossdk::bcosclientconfig::ClientConfig;
use log::info;
use crate::bcossdk::cli_common::{Cli};
use structopt::StructOpt;

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let cli:Cli = Cli::from_args();
    info!("start with cli {:?}",&cli);
    println!("console input {:?}",&cli);
    if cli.verbos > 0{
        bcossdk::macrodef::set_debugprint(true);
    }
    //println!("console cmd params: {:?}",cmdparams);


    let configfile = cli.default_configfile();

    match cli.cmd.as_str() {
        "helloworld"=>{
            println!("ready to go the  demo: helloworld contract");
            sample::helloworld::demo(configfile.as_str());
        },
        "simpleinfo"=>{
            println!("ready to go the  demo: simpleinfo contract ");
            sample::simpleinfo::demo(configfile.as_str());
        },
        "needinit"=>{
            println!("ready to go the  demo:simpleinfo contract ");
            sample::needinit::demo(configfile.as_str());
        },
        "arraydemo"=>{
            println!("ready to go the  demo : query");
            sample::arraydemo::demo(&cli);
        }
        "demoquery"=>{
            println!("ready to go the  demo : query");
            bcossdk::bcossdkquery::demo_query();
        }

        "deploy"=>{
            println!("deploy contract ");
            let res = console_contract::deploy(&cli);
            println!("{:?}",res)
        }
        "sendtx"=>{

            let res = console_contract::sendtx(&cli);
            println!("send tx result : {:?}",res);
        }
        "call"=>{
            let res = console_contract::call(&cli);
            println!("call contract result : {:?}",res);

        }
        "compile"=>{

            let res = console_contract::compile(&cli);
            println!("compile contract result : {:?}",res);

        }

        "demogmsign"=>{
            bcossdk::commonsigner::test_gm_sign();
        }

        "checkgm"=>{
            sample::checkgm::demo();
        }

        "account"=>{
            let result = console_account::cmd_account(&cli);
            println!("account cmd reuslt {:?}",result);
        }
        "usage"=>{
            console::usage::usage(&cli);
        }
        "group"=>{
            let res = sample::groupdemo::demo(&cli);
        }
        _=>{
            let res = console_cmds::handle_cmd(&cli);
            println!("console cmd result : {:?}",res);
        }
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    //set RUST_TEST_NOCAPTURE=1
    #[test]
    fn cli_check() {
           let cli:Cli = Cli::from_args();
            println!("cli {:?}",cli);
            println!("params {:?}",std::env::args_os());
            assert!(cli.cmd.len() > 0);
    }
}