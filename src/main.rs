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
mod bcossdkutil;
mod console;
mod sample;

use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use colored::Colorize;
use std::env::Args;
use std::{env, thread};

use crate::console::console_bcos2_query::Bcos2Query;
use crate::console::console_bcos3_contracts::Bcos3Contract;
use crate::console::console_bcos3_query::Bcos3Query;
use crate::console::console_cmdmap;
use crate::console::{console_account, console_bcos2_contract, console_compile};
use crate::console_bcos2_contract::Bcos2Contract;
use crate::sample::demo_bcos3event;
use console::cli_common::Cli;
use fisco_bcos_rust_gears_sdk::bcos2sdk;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcos2client::Bcos2Client;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcos_channel_tassl_sock_ffi;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcos_channel_threads_worker;
use fisco_bcos_rust_gears_sdk::bcos2sdk::eventhandler;
use fisco_bcos_rust_gears_sdk::bcos2sdk::{bcos_ssl_native, bcossdkquery};
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkwrapper;
use fisco_bcos_rust_gears_sdk::bcossdkutil::bcosclientconfig::ClientConfig;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contractabi::ContractABI;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contracthistory::ContractHistory;
use fisco_bcos_rust_gears_sdk::bcossdkutil::macrodef::set_debugprint;
use log::info;
use std::time::Duration;
use structopt::StructOpt;

#[tokio::main]
pub async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let mut cli: Cli = Cli::from_args();
    info!("start with cli {:?}", &cli);
    println!("console input {:?}", &cli);
    if cli.verbos > 0 {
        bcossdkutil::macrodef::set_debugprint(true);
    }
    //println!("console cmd params: {:?}",cmdparams);

    let configfile = cli.default_configfile();

    match cli.cmd.as_str() {
        "helloworld" => {
            println!("ready to go the  demo: helloworld contract");
            sample::helloworld::demo(configfile.as_str());
        }
        "simpleinfo" => {
            println!("ready to go the  demo: simpleinfo contract ");
            sample::simpleinfo::demo(configfile.as_str());
        }
        "needinit" => {
            println!("ready to go the  demo:simpleinfo contract ");
            sample::needinit::demo(configfile.as_str());
        }
        "arraydemo" => {
            println!("ready to go the  demo : query");
            sample::arraydemo::demo(&cli);
        }
        "demoquery" => {
            println!("ready to go the  demo : query");
            bcos2sdk::bcossdkquery::demo_query();
        }

        "compile" => {
            let res = console_compile::console_compile(&cli);
            println!("compile contract done!");
        }

        "demogmsign" => {
            bcossdkutil::commonsigner::test_gm_sign();
        }

        "checkgm" => {
            sample::checkgm::demo();
        }

        "account" => {
            let result = console_account::cmd_account(&cli);
            println!("account cmd reuslt {:?}", result);
        }
        "usage" => {
            console::usage::usage(&cli);
        }
        "structdemo" => {
            let res = sample::structdemo::demo(&cli);
        }
        "worker" => {
            println!("ready to start worker");
            set_debugprint(true);
            bcos_channel_threads_worker::start(cli.default_configfile().as_str()).await;
        }
        "event_demo" => {
            let res = eventhandler::event_demo(cli.default_configfile().as_str()).await;
        }
        "ssock_ffi" => {
            bcos_channel_tassl_sock_ffi::test_ssock();
        }

        "ssock_native" => {
            bcos_ssl_native::test_ssl_native();
        }
        "bcos3get" => {
            sample::bcos3rpc::test_bcos3sdk();
        }
        "bcos3tx" => {
            sample::bcos3tx::test_bcos3tx();
        }
        "bcos3client" => {
            sample::demo_bcos3client::demo_bcos3client(cli).unwrap();
        }
        "test_toml" => {
            bcossdkutil::contracthistory::test_toml();
        }

        "bcos2" => {
            let bcos2query = Bcos2Query::new();
            let bcos2contract = Bcos2Contract::new();
            println!("{}", "\n>---BCOS2 console---<\n".green());
            if cli.params.len() == 0 {
                println!("{}", "-->!! NO Enough params !!<<--".red());
                println!("Input: bcos3 [cmd] [params]");
                println!("eg:cargo run  bcos 3 getBlockByNumber 5");
                bcos2query.cmdmap.print_cmds(true);
                bcos2contract.cmdmap.print_cmds(true);
                return;
            }
            let cmd = cli.params.get(0).unwrap().clone();
            cli.params.remove(0);
            cli.cmd = cmd.clone();

            if bcos2query.cmdmap.in_cmd(cmd.as_str()) {
                let r = bcos2query.cmdmap.handle_cmd(&cli);
                if r.is_err() {
                    println!("console : {:?}", r);
                }
            } else if bcos2contract.cmdmap.in_cmd(cmd.as_str()) {
                let r = bcos2contract.cmdmap.handle_cmd(&cli);
                if r.is_err() {
                    println!("console : {:?}", r);
                }
            } else {
                bcos2query.cmdmap.print_cmds(true);
                bcos2contract.cmdmap.print_cmds(true);
                return;
            }
        }
        "bcos3" => {
            let bcos3query = Bcos3Query::new();
            let bcos3contract = Bcos3Contract::new();
            println!("{}", "\n>---BCOS3 console---<\n".yellow());
            if cli.params.len() == 0 {
                println!("{}", "-->!! NO Enough params !!<<--".red());
                println!("Input: bcos3 [cmd] [params]");
                println!("eg:cargo run  bcos 3 getBlockByNumber 5");
                bcos3query.cmdmap.print_cmds(true);
                bcos3contract.climap.print_cmds(true);
                return;
            }
            let cmd = cli.params.get(0).unwrap().clone();
            cli.params.remove(0);
            cli.cmd = cmd.clone();

            if bcos3query.cmdmap.in_cmd(cmd.as_str()) {
                let r = bcos3query.cmdmap.handle_cmd(&cli);
                if r.is_err() {
                    println!("console : {:?}", r);
                }
            } else if bcos3contract.climap.in_cmd(cmd.as_str()) {
                let r = bcos3contract.climap.handle_cmd(&cli);
                if r.is_err() {
                    println!("console : {:?}", r);
                }
            } else {
                bcos3query.cmdmap.print_cmds(true);
                bcos3contract.climap.print_cmds(true);
                return;
            }
        }
        "bcos3event" => {
            let res = demo_bcos3event::demo_event(&cli);
            println!("res {:?}", res);
        }
        _ => {
            println!("unhandle cmd {:?}", cli);
            console::usage::usage(&cli);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //set RUST_TEST_NOCAPTURE=1
    #[test]
    fn cli_check() {
        let cli: Cli = Cli::from_args();
        println!("cli {:?}", cli);
        println!("params {:?}", std::env::args_os());
        assert!(cli.cmd.len() > 0);
    }
}
