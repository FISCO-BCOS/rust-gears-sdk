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

use structopt::StructOpt;
use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::bcossdk::BcosSDK;
use std::time::Duration;
use crate::bcossdk::bcossdkquery;
use crate::bcossdk::contracthistory::ContractHistory;
use crate::console::{console_account, console_contract};
use crate::console::console_cmds;
use crate::bcossdk::bcosclientconfig::ClientConfig;
use log::info;

#[derive(StructOpt,Debug)]
#[structopt(about = "Fisco Bcos rust sdk console")]
pub struct Cli {
     /// 操作指令字，如 usage,deploy，sendtx，call，account，getXXX等.
     ///
     /// 输入 usage account/contract/get/all 查看对应的指令列表
     ///
     ///
     pub cmd: String,
     ///
     /// 当前操作的参数,根据操作命令字的不同会有所变化
     //#[structopt(parse(from_os_str))]
    pub params : Vec<String>,
    ///-c 配置文件，全路径如-c conf/config.toml
    #[structopt(short = "c", long = "config") ]
    pub configfile : Option<String>,
    ///-n 显式的指定合约名，不用带后缀，如"HelloWorld"
    #[structopt(short = "n", long = "contractname")]
    pub contractname : Option<String>,
    ///-v -vv -vvv...打开详细的打印
    #[structopt(short = "v",parse(from_occurrences))]
    pub verbos : u32,
}

impl  Cli{
    pub fn default_configfile(&self)->String{
        let configfile = match &self.configfile{
            Option::None =>{"conf/config.toml"},
            Some(f)=>{f.as_str()}
        };
        configfile.to_string()
    }
    pub fn default_config(&self)->Result<ClientConfig,KissError>{
        let configfile =self.default_configfile();
        ClientConfig::load(configfile.as_str())
    }

}




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
        _=>{
            let res = console_cmds::handle_cmd(&cli);
            println!("console cmd result : {:?}",res);
        }
    }

}
