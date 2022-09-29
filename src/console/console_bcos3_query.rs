use crate::bcossdkutil::liteutils;
use crate::cmdmap;
use crate::console::cli_common::Cli;
use crate::console::console_utils::{
    cli_groupid, display_transaction, display_transaction_receipt, find_contract,
    is_deploy_address, param_at, param_int,
};
use crate::console_cmdmap::CliCmdMap;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcossdkquery::json_hextoint;
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3client::Bcos3Client;
use fisco_bcos_rust_gears_sdk::bcossdkutil::kisserror::{KissErrKind, KissError};
use fisco_bcos_rust_gears_sdk::bcossdkutil::liteutils::get_opt_str;
use serde::de::Unexpected::Option as JsonOption;
use serde_json::Value as JsonValue;
use std::str::FromStr;

pub struct Bcos3Query {
    pub cmdmap: CliCmdMap,
}
impl Bcos3Query {
    pub fn new() -> Self {
        let mut query = Bcos3Query {
            cmdmap: CliCmdMap::new("BCOS3 Query"),
        };
        //用宏定义实现将方法加到map里，宏里可以自动识别方法的名字，作为map的key
        //于是后续调用时，用名字去match就行了,且可以做到大小写不敏感
        cmdmap!(query.cmdmap.cmd_func_map, getVersion);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockLimit);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockNumber);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockByNumber);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockByHash);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockHashByNumber);
        cmdmap!(query.cmdmap.cmd_func_map, getTransactionByHash);
        cmdmap!(query.cmdmap.cmd_func_map, getTransactionReceipt);
        cmdmap!(query.cmdmap.cmd_func_map, getPbftView);
        cmdmap!(query.cmdmap.cmd_func_map, getSealerList);
        cmdmap!(query.cmdmap.cmd_func_map, getObserverList);
        cmdmap!(query.cmdmap.cmd_func_map, getSyncStatus);
        cmdmap!(query.cmdmap.cmd_func_map, getConsensusStatus);
        cmdmap!(query.cmdmap.cmd_func_map, getPeers);
        cmdmap!(query.cmdmap.cmd_func_map, getGroupPeers);
        cmdmap!(query.cmdmap.cmd_func_map, getPendingTxSize);
        cmdmap!(query.cmdmap.cmd_func_map, getCode);
        cmdmap!(query.cmdmap.cmd_func_map, getTotalTransactionCount);
        cmdmap!(query.cmdmap.cmd_func_map, getSystemConfigByKey);

        query
    }
}

pub fn getVersion(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let v = bcossdk.getVersion()?;
    println!("{}\n", v);
    Ok(())
}

pub fn getBlockLimit(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let v = bcossdk.getBlocklimit()?;
    println!("\n {:?}\n", v);
    Ok(())
}

pub fn getBlockNumber(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let v = bcossdk.getBlockNumber()?;
    println!("\n {:?}\n", v);
    Ok(())
}

pub fn getBlockHashByNumber(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let num = u64::from_str_radix(cli.params[0].as_str(), 10).unwrap();
    let v = bcossdk.getBlockHashByNumber(num)?;
    println!("\n {:?}\n", v);

    Ok(())
}

pub fn getBlockByNumber(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let num = u64::from_str_radix(cli.params[0].as_str(), 10).unwrap();
    let mut only_header = 0;
    if cli.params.len() > 1 {
        only_header = u32::from_str_radix(cli.params[1].as_str(), 10).unwrap();
    }
    let mut only_tx_hash = 1;
    if cli.params.len() > 2 {
        only_tx_hash = u32::from_str_radix(cli.params[2].as_str(), 10).unwrap();
    }
    let v = bcossdk.getBlockByNumber(num, only_header, only_tx_hash)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockByHash(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let hash = cli.params[0].as_str();
    let mut only_header = 0;
    if cli.params.len() > 1 {
        only_header = u32::from_str_radix(cli.params[1].as_str(), 10).unwrap();
    }
    let mut only_tx_hash = 1;
    if cli.params.len() > 2 {
        only_tx_hash = u32::from_str_radix(cli.params[2].as_str(), 10).unwrap();
    }
    let v = bcossdk.getBlockByHash(hash, only_header, only_tx_hash)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getTransactionByHash(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let cmd = "getTransaction";
    let hash = cli.params[0].as_str();
    let mut proof = 1;
    if cli.params.len() > 1 {
        proof = u32::from_str_radix(cli.params[2].as_str(), 10).unwrap();
    }
    let v = bcossdk.getTransactionByHash(hash, proof as i32)?;
    let res = display_transaction(
        &v,
        &bcossdk.config,
        bcossdk.get_full_name().as_str(),
        get_opt_str(&cli.contractname).as_str(),
    );
    // display the receipt of transaction
    let hash = v["hash"].as_str().unwrap();
    let receipt = bcossdk.getTransactionReceipt(hash, proof as i32)?;
    let to = v["to"].as_str().unwrap();
    println!("->try find contract for {}", bcossdk.get_full_name());
    let contractABI = find_contract(
        bcossdk.get_full_name().as_str(),
        get_opt_str(&cli.contractname).as_str(),
        to,
        &bcossdk.config,
    )?;
    println!("->try display receipt");
    display_transaction_receipt(&receipt, &Option::from(&contractABI), &bcossdk.config);
    Ok(())
}

pub fn getTransactionReceipt(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let cmd = "getTransactionReceipt";
    let hash = cli.params[0].as_str();
    let mut proof = 1;
    if cli.params.len() > 1 {
        proof = u32::from_str_radix(cli.params[2].as_str(), 10).unwrap();
    }

    let v = bcossdk.getTransactionReceipt(hash, proof as i32)?;
    println!(
        "\n[{}] : {}\n",
        cmd,
        serde_json::to_string_pretty(&v).unwrap()
    );
    if v == JsonValue::Null {
        println!("reuslt is Null");
        return Ok(());
    }
    let to = v["to"].as_str().unwrap();
    if is_deploy_address(to) {
        let blocknum = json_hextoint(&v["blockNumber"]).unwrap();
        println!("is a deploy contract transaction on block [{}]", blocknum);
        return Ok(());
    }

    let contractres = find_contract(
        bcossdk.get_full_name().as_str(),
        get_opt_str(&cli.contractname).as_str(),
        to,
        &bcossdk.config,
    );
    let contract = match contractres {
        Ok(c) => c,
        Err(e) => {
            //miss ,but maybe not a error
            println!(
                "Missing contract history for address  [{}] ,but ok &done ",
                to
            );
            return Ok(());
        }
    };
    display_transaction_receipt(&v, &Option::from(&contract), &bcossdk.config);
    Ok(())
}

pub fn getPbftView(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPbftView()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getSealerList(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getSealerList()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getObserverList(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getObserverList()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getSyncStatus(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getSyncStatus()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getConsensusStatus(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getConsensusStatus()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getPeers(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPeers()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getGroupPeers(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getGroupPeers()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getGroupList(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    // let groupid =cli_groupid(&cli);
    let v = bcossdk.getGroupList()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getPendingTxSize(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPendingTxSize()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getTotalTransactionCount(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getTotalTransactionCount()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());

    let blockNumber = liteutils::json_u64(&v, "blockNumber", -1);
    let failedTxSum = liteutils::json_u64(&v, "failedTxSum", -1);
    let txSum = liteutils::json_u64(&v, "txSum", -1);
    println!(
        "blockNumber:{},txSum:{},failedTxSum:{}",
        blockNumber, txSum, failedTxSum
    );
    Ok(())
}

pub fn getCode(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let address = param_at(&cli.params, 1)?;
    let v = bcossdk.getCode(address.as_str())?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getSystemConfigByKey(cli: &Cli) -> Result<(), KissError> {
    let bcossdk = Bcos3Client::new(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let key = param_at(&cli.params, 1)?;
    let v = bcossdk.getSystemConfigByKey(key.as_str())?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}
