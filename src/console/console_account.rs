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
use rustc_hex::ToHex;
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

pub fn cmd_account(params: &Vec<String>, configfile: &str) -> Result<(), KissError> {
    let configfilepath = PathBuf::from(&configfile);
    let mut workpath = configfilepath.clone();
    workpath.pop();
    let config = ClientConfig::load(configfile)?;
    let opt: OptAccount = StructOpt::from_iter(params.iter());
    //println!("{:?}",opt);
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
    let newaccount = create_account(&config.chain.crypto);
    println!(">>> create new account --> \n{}", newaccount.to_hexdetail());
    let mut fullpath = PathBuf::from(workpath);
    match name {
        Some(n) => {
            fullpath = fullpath.join(format!("{}.pem", n));
        }
        Option::None => {
            fullpath = fullpath.join(format!("{}.pem", newaccount.address.to_hex()));
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
use crate::bcossdk;
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
            return show_account_from_pem(path.to_str().unwrap(), &config.chain.crypto);
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
                    show_account_from_pem(entry.path().to_str().unwrap(), &config.chain.crypto)?
                }
            }
            _ => {}
        }
    }
    Ok(())
}
