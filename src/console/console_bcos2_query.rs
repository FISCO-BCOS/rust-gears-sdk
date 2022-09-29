use crate::cmdmap;
use crate::console::cli_common::Cli;
use crate::console::console_utils::{
    cli_groupid, display_transaction, display_transaction_receipt, find_contract,
    is_deploy_address, param_at, param_int,
};
use crate::console_cmdmap::CliCmdMap;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcos2client::Bcos2Client;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcossdkquery::json_hextoint;
use fisco_bcos_rust_gears_sdk::bcossdkutil::kisserror::{KissErrKind, KissError};
use fisco_bcos_rust_gears_sdk::bcossdkutil::liteutils::get_opt_str;
use serde::de::Unexpected::Option as JsonOption;
use serde_json::Value as JsonValue;
use std::str::FromStr;

pub struct Bcos2Query {
    pub cmdmap: CliCmdMap,
}
impl Bcos2Query {
    pub fn new() -> Self {
        let mut query = Bcos2Query {
            cmdmap: CliCmdMap::new("BCOS2 Query"),
        };
        //用宏定义实现将方法加到map里，宏里可以自动识别方法的名字，作为map的key
        //于是后续调用时，用名字去match就行了,且可以做到大小写不敏感
        cmdmap!(query.cmdmap.cmd_func_map, getBlockNumber);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockByNumber);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockByHash);
        cmdmap!(query.cmdmap.cmd_func_map, getTransactionByHash);
        cmdmap!(query.cmdmap.cmd_func_map, getTransactionByHash);
        cmdmap!(query.cmdmap.cmd_func_map, getTransactionReceipt);
        cmdmap!(
            query.cmdmap.cmd_func_map,
            getTransactionByBlockNumberAndIndex
        );
        cmdmap!(query.cmdmap.cmd_func_map, getTransactionByBlockHashAndIndex);
        cmdmap!(query.cmdmap.cmd_func_map, getPbftView);
        cmdmap!(query.cmdmap.cmd_func_map, getSealerList);
        cmdmap!(query.cmdmap.cmd_func_map, getObserverList);
        cmdmap!(query.cmdmap.cmd_func_map, getSyncStatus);
        cmdmap!(query.cmdmap.cmd_func_map, getConsensusStatus);
        cmdmap!(query.cmdmap.cmd_func_map, getPeers);
        cmdmap!(query.cmdmap.cmd_func_map, getNodeVersion);
        cmdmap!(query.cmdmap.cmd_func_map, getGroupPeers);
        cmdmap!(query.cmdmap.cmd_func_map, getGroupList);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockHeaderByHash);
        cmdmap!(query.cmdmap.cmd_func_map, getBlockHeaderByNumber);
        cmdmap!(query.cmdmap.cmd_func_map, getPendingTransactions);
        cmdmap!(query.cmdmap.cmd_func_map, getPendingTxSize);
        cmdmap!(query.cmdmap.cmd_func_map, getCode);
        cmdmap!(query.cmdmap.cmd_func_map, getTotalTransactionCount);
        cmdmap!(query.cmdmap.cmd_func_map, getSystemConfigByKey);
        cmdmap!(query.cmdmap.cmd_func_map, queryGroupStatus);

        query
    }
}

pub fn getNodeVersion(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let v = bcossdk.getNodeVersion()?;
    println!("{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockNumber(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let v = bcossdk.getBlockNumber()?;
    println!("\n {:?}\n", v);
    Ok(())
}

pub fn getBlockByNumber(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let num = u32::from_str_radix(cli.params[0].as_str(), 10).unwrap();
    let mut includeTransactions = true;
    if cli.params.len() > 1 {
        includeTransactions = bool::from_str(cli.params[1].as_str()).unwrap();
    }
    let v = bcossdk.getBlockByNumber(num, includeTransactions)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockByHash(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let hash = cli.params[0].as_str();
    let mut includeTransactions = true;
    if cli.params.len() > 1 {
        includeTransactions = bool::from_str(cli.params[1].as_str()).unwrap();
    }
    let v = bcossdk.getBlockByHash(hash, includeTransactions)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockHeaderByHash(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let hash = cli.params[0].as_str();
    let mut includeTransactions = true;
    if cli.params.len() > 1 {
        includeTransactions = bool::from_str(cli.params[1].as_str()).unwrap();
    }
    let v = bcossdk.getBlockHeaderByHash(hash, includeTransactions)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getBlockHeaderByNumber(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let num = param_int(&cli.params, 0)?;
    let mut includeTransactions = true;
    if cli.params.len() > 1 {
        includeTransactions = bool::from_str(cli.params[1].as_str()).unwrap();
    }
    let v = bcossdk.getBlockHeaderByNumber(num as u32, includeTransactions)?;
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getTransactionByHash(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let cmd = "getTransactionByHash";
    let hash = cli.params[0].as_str();
    let v = bcossdk.getTransactionByHash(hash)?;
    let res = display_transaction(
        &v["result"],
        &bcossdk.config,
        "bcos2",
        get_opt_str(&cli.contractname).as_str(),
    );
    // display the receipt of transaction
    let hash = v["result"]["hash"].as_str().unwrap();
    let receipt = bcossdk.getTransactionReceipt(hash)?;
    let to = v["result"]["to"].as_str().unwrap();
    let contractABI = find_contract(
        "bcos2",
        get_opt_str(&cli.contractname).as_str(),
        to,
        &bcossdk.config,
    )?;
    display_transaction_receipt(&receipt, &Option::from(&contractABI), &bcossdk.config);
    Ok(())
}

pub fn getTransactionByBlockNumberAndIndex(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let cmd = "getTransactionByBlockNumberAndIndex";
    let num = u32::from_str_radix(cli.params[0].as_str(), 10).unwrap();
    let index = u32::from_str_radix(cli.params[1].as_str(), 10).unwrap();
    let v = bcossdk.getTransactionByBlockNumberAndIndex(num, index)?;
    let cli_contractname = get_opt_str(&cli.contractname);
    let res = display_transaction(
        &v["result"],
        &bcossdk.config,
        "bcos2",
        get_opt_str(&cli.contractname).as_str(),
    );

    // display the receipt of transaction
    let hash = v["result"]["hash"].as_str().unwrap();
    let receipt = bcossdk.getTransactionReceipt(hash)?;
    let to = v["result"]["to"].as_str().unwrap();
    let contractABI = find_contract(
        "bcos2",
        get_opt_str(&cli.contractname).as_str(),
        to,
        &bcossdk.config,
    )?;
    display_transaction_receipt(&receipt, &Option::from(&contractABI), &bcossdk.config);
    Ok(())
}

pub fn getTransactionByBlockHashAndIndex(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let cmd = "getTransactionByBlockNumberAndIndex";
    let blockhash = cli.params[0].as_str();
    let index = u32::from_str_radix(cli.params[1].as_str(), 10).unwrap();
    let v = bcossdk.getTransactionByBlockHashAndIndex(blockhash, index)?;
    let res = display_transaction(
        &v["result"],
        &bcossdk.config,
        "bcos2",
        get_opt_str(&cli.contractname).as_str(),
    );

    // display the receipt of transaction
    let hash = v["result"]["hash"].as_str().unwrap();
    let receipt = bcossdk.getTransactionReceipt(hash)?;
    let to = v["result"]["to"].as_str().unwrap();
    let contract = find_contract(
        "bcos2",
        get_opt_str(&cli.contractname).as_str(),
        to,
        &bcossdk.config,
    )?;
    display_transaction_receipt(&receipt, &Option::from(&contract), &bcossdk.config);

    Ok(())
}

pub fn getTransactionReceipt(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let cmd = "getTransactionReceipt";
    let hash = cli.params[0].as_str();
    let v = bcossdk.getTransactionReceipt(hash)?;
    println!(
        "\n[{}] : {}\n",
        cmd,
        serde_json::to_string_pretty(&v).unwrap()
    );
    if v["result"] == JsonValue::Null {
        println!("reuslt is Null");
        return Ok(());
    }
    let to = v["result"]["to"].as_str().unwrap();
    if is_deploy_address(to) {
        let blocknum = json_hextoint(&v["result"]["blockNumber"]).unwrap();
        println!("is a deploy contract transaction on block [{}]", blocknum);
        return Ok(());
    }

    let contractres = find_contract(
        "bcos2",
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
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPbftView(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getSealerList(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getSealerList(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getObserverList(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getObserverList(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getSyncStatus(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getSyncStatus(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getConsensusStatus(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getConsensusStatus(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getPeers(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPeers(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getGroupPeers(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getGroupPeers(groupid as u32)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getGroupList(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    // let groupid =cli_groupid(&cli);
    let v = bcossdk.getGroupList()?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getPendingTransactions(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPendingTransactions(groupid)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}
pub fn getPendingTxSize(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getPendingTxSize(groupid)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getTotalTransactionCount(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.getTotalTransactionCount(groupid)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());

    let blockNumber = json_hextoint(&v["result"]["blockNumber"])?;
    let failedTxSum = json_hextoint(&v["result"]["failedTxSum"])?;
    let txSum = json_hextoint(&v["result"]["txSum"])?;
    println!(
        "blockNumber:{},txSum:{},failedTxSum:{}",
        blockNumber, txSum, failedTxSum
    );
    Ok(())
}

pub fn getCode(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let address = param_at(&cli.params, 1)?;
    let v = bcossdk.getCode(groupid, address.as_str())?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}

pub fn getSystemConfigByKey(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let key = param_at(&cli.params, 1)?;
    let v = bcossdk.getSystemConfigByKey(groupid, key.as_str())?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}
pub fn queryGroupStatus(cli: &Cli) -> Result<(), KissError> {
    let mut bcossdk = Bcos2Client::new_from_config(cli.default_configfile().as_str())?;
    let groupid = cli_groupid(&cli);
    let v = bcossdk.queryGroupStatus(groupid)?;
    println!("{}", serde_json::to_string_pretty(&v).unwrap());
    Ok(())
}
