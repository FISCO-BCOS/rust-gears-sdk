#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals
)]
use crate::bcossdk::accountutil::{
    account_from_pem, create_account, save_key_to_pem, BcosAccount, EcdsaAccountUtil,
    GMAccountUtil, IBcosAccountUtil,
};
use crate::bcossdk::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use crate::bcossdk::kisserror::KissError;
use hex::ToHex;
use std::path::PathBuf;
use structopt::StructOpt;

/// bcos_rust_sdk account new alice
/// bcos_rust_sdk account show alice/none for all
#[derive(StructOpt, Debug)]
#[structopt(about = "account new&show")]
struct OptAccount {
    operation: String,
    name: Option<String>,
}

pub fn cmd_account(cli:&Cli) -> Result<(), KissError> {

        //将cmd和param拼在一起，作为新的args，给到StructOpt去解析（因为第一个参数总是app名）
    let mut cmdparams :Vec<String>= vec!(cli.cmd.clone());

    cmdparams.append(&mut cli.params.clone());
    println!("cmdparams {:?}",cmdparams);
    let configfile = cli.default_configfile();

    let configfilepath = PathBuf::from(&configfile);
    let mut workpath = configfilepath.clone();
    workpath.pop();
    let config = ClientConfig::load(configfile.as_str())?;
    let opt: OptAccount = StructOpt::from_iter(cmdparams.iter());
    println!("{:?}",opt);
    match opt.operation.as_str() {
        "new" => {
            return newaccount(&opt.name, &config, workpath.to_str().unwrap());
        }
        "show" => {
            let res = showaccount(&opt.name, &config, workpath.to_str().unwrap());
            println!("show account {:?}", res);
        }
        _ => {}
    }
    Ok(())
}

pub fn newaccount(
    name: &Option<String>,
    config: &ClientConfig,
    workpath: &str,
) -> Result<(), KissError> {
    let newaccount = create_account(&config.common.crypto);
    println!(">>> create new account --> \n{}", newaccount.to_hexdetail());
    let mut fullpath = PathBuf::from(workpath);
    match name {
        Some(n) => {
            fullpath = fullpath.join(format!("{}.pem", n));
        }
        Option::None => {
            fullpath = fullpath.join(format!("{}.pem",hex::encode( &newaccount.address)));
        }
    }
    println!("new account save to : {}", fullpath.to_str().unwrap());
    save_key_to_pem(&newaccount.privkey, fullpath.as_path().to_str().unwrap())
}

pub fn show_account_from_pem(path: &str, cryptokind: &BcosCryptoKind) -> Result<(), KissError> {
    println!(
        "\n>>> load acccount from {} ,crypto : {:?}",
        path, cryptokind
    );
    let account = account_from_pem(path, cryptokind)?;

    println!("{}", account.to_hexdetail());
    Ok(())
}
use crate::{bcossdk, Cli};
use std::ffi::OsStr;
use std::fs;

pub fn showaccount(
    name: &Option<String>,
    config: &ClientConfig,
    workpath: &str,
) -> Result<(), KissError> {
    // bcossdk::macrodef::set_debugprint(true);
    //println!("name {:?}",name);
    match name {
        Some(n) => {
            let mut path = PathBuf::from(workpath);
            path = path.join(format!("{}.pem", n));
            //println!("path is {:?}",path);
            return show_account_from_pem(path.to_str().unwrap(), &config.common.crypto);
        }
        _ => {}
    }

    let paths = fs::read_dir(workpath).unwrap();
    for path in paths {
        let entry = path.unwrap();
        //println!("Name: {}", &entry.path().display());
        //println!("extension {:?}",entry.path().extension().unwrap().to_str());
        let p = entry.path();
        let extstr = p.extension();
        match extstr {
            Some(ss) => {
                let exts = ss.to_str().unwrap();
                if exts == "pem" {
                    show_account_from_pem(entry.path().to_str().unwrap(), &config.common.crypto)?
                }
            }
            _ => {}
        }
    }
    Ok(())
}
