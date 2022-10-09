use fisco_bcos_rust_gears_sdk::bcossdkutil::contractabi::ContractABI;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contracthistory::ContractHistory;
use fisco_bcos_rust_gears_sdk::bcossdkutil::kisserror::{KissErrKind, KissError};

use crate::bcossdkutil::liteutils;
use crate::console::cli_common::OptContract;
use crate::console::console_compile::console_compile;
use crate::console_cmdmap::CliCmdMap;
use crate::kisserr;
use crate::{cmdmap, Cli};
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcossdkquery::json_hextoint;
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3client::Bcos3Client;
use fisco_bcos_rust_gears_sdk::bcossdkutil::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use fisco_bcos_rust_gears_sdk::bcossdkutil::solcompile::sol_compile;
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

pub struct Bcos3Contract {
    pub climap: CliCmdMap,
}
impl Bcos3Contract {
    pub fn new() -> Self {
        let mut cmdhandler = Bcos3Contract {
            climap: CliCmdMap::new("BCOS3 Contract"),
        };
        //用宏定义实现将方法加到map里，宏里可以自动识别方法的名字，作为map的key
        //于是后续调用时，用名字去match就行了,且可以做到大小写不敏感
        cmdmap!(cmdhandler.climap.cmd_func_map, deploy);
        cmdmap!(cmdhandler.climap.cmd_func_map, call);
        cmdmap!(cmdhandler.climap.cmd_func_map, sendtx);
        cmdhandler
    }
}

pub fn deploy(cli: &Cli) -> Result<(), KissError> {
    let configfile = cli.default_configfile();
    let bcos3client = Bcos3Client::new(configfile.as_str())?;
    println!("{}",bcos3client.get_info());
    println!("-------------------------------------");
    //每次部署前强制编译一次对应合约，考虑到合约sol可能会有修改
    let res = console_compile(cli)?;

    let contractname = &cli.params[0];

    let params = &cli.params[1..];
    println!("deploy contract {} ,params:{:?}", contractname, params);

    let binfile = format!(
        "{}/{}.bin",
        bcos3client.config.common.contractpath,
        contractname.to_string()
    );
    let res = bcos3client.deploy_withparam(contractname.as_str(), &params)?;
    //println!("deploy transaction return :{:?}",&res);
    let hash = res["contractAddress"].as_str().unwrap().clone().to_string();

    let address = res["contractAddress"].as_str().unwrap();
    let blocknum = liteutils::json_u64(&res, "blockNumber", -1);
    println!(
        "deploy contract on block[{}], address is {}",
        blocknum, address
    );
    let chf = ContractHistory::history_file(bcos3client.config.common.contractpath.as_str());
    let res = ContractHistory::save_to_file(
        chf.as_str(),
        bcos3client.get_full_name().as_str(),
        contractname,
        address,
        blocknum as u64,
    );
    println!("save contract history to file {} ,{:?}", chf, res);
    Ok(())
}
pub fn sendtx(cli: &Cli) -> Result<(), KissError> {
    let configfile = cli.default_configfile();

    let bcos3client = Bcos3Client::new(configfile.as_str())?;
    println!("{}",bcos3client.get_info());
    println!("-------------------------------------");
    //将cmd和param拼在一起，作为新的args，给到StructOpt去解析（因为第一个参数总是app名）
    let mut cmdparams: Vec<String> = vec![cli.cmd.clone()];
    cmdparams.append(&mut cli.params.clone());
    let opt: OptContract = StructOpt::from_iter(cmdparams.iter());

    let contractdir = "contracts";
    let contractfullname = format!("{}/{}.abi", contractdir, &opt.contract_name);
    println!("contract file is {}", contractfullname);
    let contract = ContractABI::new(contractfullname.as_str(), &bcos3client.hashtype)?;
    let chfile = format!(
        "{}/contracthistory.toml",
        bcos3client.config.common.contractpath
    );
    let address = ContractHistory::check_address_from_file(
        chfile.as_str(),
        bcos3client.get_full_name().as_str(),
        opt.contract_name.as_str(),
        opt.address.as_str(),
    )?;

    println!("contract address is {}", &address.as_str());
    println!("method is {}", opt.method);
    let response = bcos3client.sendTransaction(
        address.as_str(),
        opt.method.as_str(),
        &opt.params,
        &contract,
    )?;
    //println!("send_raw_transaction result {:?}", response);
    println!("response[\"result\"] {:?}", response);
    let txhash = response["transactionHash"].as_str().unwrap();
    println!("\n>>>>>>>>>>>>>>>>>>>after sendtx getTransactionByHash");
    let receiptresult = bcos3client.getTransactionReceipt(txhash, 0);

    match receiptresult {
        Ok(receipt) => {
            crate::console::console_utils::display_transaction_receipt(
                &receipt,
                &Option::from(&contract),
                &bcos3client.config,
            );
        }
        Err(e) => return kisserr!(KissErrKind::ENetwork, "{:?}", e),
    };
    /*
    let txdata = bcossdkutil.getTransactionByHash(txhash).unwrap();
    let blocknum = bcossdkquery::json_hextoint(&txdata["blockNumber"]);
    let txinput = txdata["input"].as_str().unwrap();
    let inputdecode = contract.decode_input_for_tx(txinput).unwrap();
    println!("tx input :{:?}",inputdecode);*/
    Ok(())
}

pub fn call(cli: &Cli) -> Result<(), KissError> {
    let configfile = cli.default_configfile();

    let bcos3client = Bcos3Client::new(configfile.as_str())?;
    println!("{}",bcos3client.get_info());
    println!("-------------------------------------");

    //将cmd和param拼在一起，作为新的args，给到StructOpt去解析（因为第一个参数总是app名）
    let mut cmdparams: Vec<String> = vec![cli.cmd.clone()];
    cmdparams.append(&mut cli.params.clone());
    let opt: OptContract = StructOpt::from_iter(cmdparams.iter());
    let contractdir = "contracts";
    let contractfullname = format!("{}/{}.abi", contractdir, &opt.contract_name);
    println!("contract file is {}", contractfullname);
    let contract = ContractABI::new(contractfullname.as_str(), &bcos3client.hashtype)?;
    let chfile = format!(
        "{}/contracthistory.toml",
        bcos3client.config.common.contractpath
    );
    let address = ContractHistory::check_address_from_file(
        chfile.as_str(),
        bcos3client.get_full_name().as_str(),
        opt.contract_name.as_str(),
        opt.address.as_str(),
    )?;

    println!("contract address is {}", &address.as_str());
    let res = bcos3client.call(
        address.as_str(),
        opt.method.as_str(),
        &opt.params,
        &contract,
    )?;
    println!(
        "call result :{}",
        serde_json::to_string_pretty(&res).unwrap()
    );

    let ustatus = res["status"].as_i64().unwrap();
    println!("call status code ({:?}) ", ustatus);
    if ustatus == 0 {
        let output = res["output"].as_str().unwrap();
        let decodereuslt = contract.decode_output_byname(opt.method.as_str(), output);
        println!("call output: {:?}", decodereuslt);
    } else {
        return kisserr!(KissErrKind::Error, "call error !!!");
    }
    Ok(())
}
