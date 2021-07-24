use crate::{kisserr, Cli};
use crate::bcossdk::kisserror::{KissError, KissErrKind};
use crate::bcossdk::bcossdk::BcosSDK;
use std::str::FromStr;
use serde_json::{Value as JsonValue};
use crate::bcossdk::contracthistory::ContractHistory;
use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::commonhash::CommonHash;
use ethabi::Log;
use crate::bcossdk::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use secp256k1::ffi::secp256k1_context_no_precomp;
use crate::console::console_utils::{display_transaction_receipt, display_transaction, find_contracthistory_byaddress, find_contract, is_deploy_address, param_int, cli_groupid, param_at};
use crate::bcossdk::bcossdkquery::json_hextoint;
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::stringify;

///--------------在这个文件里聚合一下查询接口,用简单的公共方法提供----------------------------------------
///方法命令刻意和https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html
///保持大小写和拼写一致，以便查找，局部不遵循rust命令规范

type CMD_FUNCS = fn(&mut BcosSDK, &Cli) -> Result<(), KissError>;
#[macro_export]
macro_rules! cmdmap {
            ($m:expr,$x:ident) => {
                $m.insert(stringify!($x).trim_start_matches("").to_lowercase() ,($x) )
            };
}
lazy_static! {
    static ref CMDS_MAP: HashMap<String,CMD_FUNCS> = {
        let mut cmap:HashMap<String,CMD_FUNCS> = HashMap::new();

        //用宏定义实现将方法加到map里，宏里可以自动识别方法的名字，作为map的key
        //于是后续调用时，用名字去match就行了,且可以做到大小写不敏感
        cmdmap!(cmap,   getBlockNumber);
        cmdmap!(cmap,   getBlockByNumber);
        cmdmap!(cmap,   getBlockByHash);
        cmdmap!(cmap,   getTransactionByHash);
        cmdmap!(cmap,   getTransactionByHash);
        cmdmap!(cmap,   getTransactionReceipt);
        cmdmap!(cmap,   getTransactionByBlockNumberAndIndex);
        cmdmap!(cmap,   getTransactionByBlockHashAndIndex);
        cmdmap!(cmap,   getPbftView);
        cmdmap!(cmap,   getSealerList);
        cmdmap!(cmap,   getObserverList);
        cmdmap!(cmap,   getSyncStatus);
        cmdmap!(cmap,   getConsensusStatus);
        cmdmap!(cmap,   getPeers);
        cmdmap!(cmap,   getNodeVersion);
        cmdmap!(cmap,   getGroupPeers);
        cmdmap!(cmap,   getGroupList);
        cmdmap!(cmap,   getBlockHeaderByHash);
        cmdmap!(cmap,   getBlockHeaderByNumber);
        cmdmap!(cmap,   getPendingTransactions);
        cmdmap!(cmap,   getPendingTxSize);
        cmdmap!(cmap,   getCode);
        cmdmap!(cmap,   getTotalTransactionCount);
        cmdmap!(cmap,   getSystemConfigByKey);
        cmdmap!(cmap,   queryGroupStatus);


        for (k,v) in cmap.iter(){
         //   println!("{:?}",k);
        }


        cmap
    };

}

pub fn handle_cmd(cli: &Cli) -> Result<(), KissError>
{
    let mut bcossdk = BcosSDK::new_from_config(cli.default_configfile().as_str())?;
    println!("BcosSDK: {}", bcossdk.to_summary());
    let cmd = &cli.cmd;
    let seekkey = cli.cmd.to_lowercase();
    if CMDS_MAP.contains_key(seekkey.as_str()) {
        let func = CMDS_MAP.get(seekkey.as_str()).unwrap();
        println!("\n{} -->",cli.cmd.as_str());
        let res = func(&mut bcossdk, &cli);
        match res {
            Ok(()) => {}
            Err(e) => { println!("cmd {} error {:?}", cmd, e) }
        }
        return Ok(());
    } else {
        println!("cmd [ {} ]not implement yet.Valid cmds: ", cmd);
        let mut i = 1;
        for (k, v) in CMDS_MAP.iter() {
            print!("\t{:02})->\t{}\n ",i, k);
            i += 1;
        }
        println!("");
        return kisserr!(KissErrKind::Error,"cmd {} not implement yet ",cmd);
    }
}

pub fn getNodeVersion(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let v = bcossdk.getNodeVersion()?;
    println!("{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockNumber(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let v = bcossdk.getBlockNumber()?;
    println!("\n {:?}\n", v);
    Ok(())
}

pub fn getBlockByNumber(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let num = u32::from_str_radix(cli.params[0].as_str(), 10).unwrap();
    let mut includeTransactions = true;
    if cli.params.len() > 1 {
        includeTransactions = bool::from_str(cli.params[1].as_str()).unwrap();
    }
    let v = bcossdk.getBlockByNumber(num, includeTransactions)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockByHash(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let hash = cli.params[0].as_str();
    let mut includeTransactions = true;
    if cli.params.len() > 1 {
        includeTransactions = bool::from_str(cli.params[1].as_str()).unwrap();
    }
    let v = bcossdk.getBlockByHash(hash, includeTransactions)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockHeaderByHash(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let hash = cli.params[0].as_str();
    let mut includeTransactions = true;
    if cli.params.len() > 1 {
        includeTransactions = bool::from_str(cli.params[1].as_str()).unwrap();
    }
    let v = bcossdk.getBlockHeaderByHash(hash, includeTransactions)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockHeaderByNumber(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let num = param_int(&cli.params, 0)?;
    let mut includeTransactions = true;
    if cli.params.len() > 1 {
        includeTransactions = bool::from_str(cli.params[1].as_str()).unwrap();
    }
    let v = bcossdk.getBlockHeaderByNumber(num as u32, includeTransactions)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}


pub fn getTransactionByHash(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let cmd = "getTransactionByHash";
    let hash = cli.params[0].as_str();
    let v = bcossdk.getTransactionByHash(hash)?;
    let res = display_transaction(&v, bcossdk, &cli);
    Ok(())
}

pub fn getTransactionReceipt(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let cmd = "getTransactionReceipt";
    let hash = cli.params[0].as_str();
    let v = bcossdk.getTransactionReceipt(hash)?;
    println!("\n[{}] : {}\n", cmd, serde_json::to_string_pretty(&v).unwrap());
    if v["result"] == JsonValue::Null
    {
        println!("reuslt is Null");
        return Ok(());
    }
    let to = v["result"]["to"].as_str().unwrap();
    if is_deploy_address(to) {
        let blocknum = json_hextoint(&v["result"]["blockNumber"]).unwrap();
        println!("is a deploy contract transaction on block [{}]", blocknum);
        return Ok(());
    }

    let contractres = find_contract(&cli.contractname, to, &bcossdk.config);
    let contract = match contractres {
        Ok(c) => { c }
        Err(e) => {
            //miss ,but maybe not a error
            println!("Missing contract history for address  [{}] ,but ok &done ", to);
            return Ok(());
        }
    };
    display_transaction_receipt(&v, &Option::from(&contract), &bcossdk.config);
    Ok(())
}

pub fn getTransactionByBlockNumberAndIndex(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let cmd = "getTransactionByBlockNumberAndIndex";
    let num = u32::from_str_radix(cli.params[0].as_str(), 10).unwrap();
    let index = u32::from_str_radix(cli.params[1].as_str(), 10).unwrap();
    let v = bcossdk.getTransactionByBlockNumberAndIndex(num, index)?;
    let res = display_transaction(&v, bcossdk, &cli);
    Ok(())
}

pub fn getTransactionByBlockHashAndIndex(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let cmd = "getTransactionByBlockNumberAndIndex";
    let blockhash = cli.params[0].as_str();
    let index = u32::from_str_radix(cli.params[1].as_str(), 10).unwrap();
    let v = bcossdk.getTransactionByBlockHashAndIndex(blockhash, index)?;
    let res = display_transaction(&v, bcossdk, &cli);
    Ok(())
}


pub fn getPbftView(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPbftView(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getSealerList(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getSealerList(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getObserverList(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getObserverList(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getSyncStatus(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getSyncStatus(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getConsensusStatus(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getConsensusStatus(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}


pub fn getPeers(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPeers(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getGroupPeers(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getGroupPeers(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getGroupList(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    // let groupid =cli_groupid(&cli);
    let v = bcossdk.getGroupList()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getPendingTransactions(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid =cli_groupid(&cli);
    let v = bcossdk.getPendingTransactions(groupid)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}
pub fn getPendingTxSize(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid =cli_groupid(&cli);
    let v = bcossdk.getPendingTxSize(groupid)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getTotalTransactionCount(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid =cli_groupid(&cli);
    let v = bcossdk.getTotalTransactionCount(groupid)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());

    let blockNumber = json_hextoint(&v["result"]["blockNumber"])?;
    let failedTxSum = json_hextoint(&v["result"]["failedTxSum"])?;
     let txSum = json_hextoint(&v["result"]["txSum"])?;
    println!("blockNumber:{},txSum:{},failedTxSum:{}",blockNumber,txSum,failedTxSum);
    Ok(())
}

pub fn getCode(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid =cli_groupid(&cli);
    let address = param_at(&cli.params,1)?;
    let v = bcossdk.getCode(groupid,address.as_str())?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}


pub fn getSystemConfigByKey(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid =cli_groupid(&cli);
    let key = param_at(&cli.params,1)?;
    let v = bcossdk.getSystemConfigByKey(groupid,key.as_str())?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}
pub fn queryGroupStatus(bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError> {
    let groupid =cli_groupid(&cli);
    let v = bcossdk.queryGroupStatus(groupid)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

