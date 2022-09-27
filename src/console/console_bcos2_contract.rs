use fisco_bcos_rust_gears_sdk::bcossdk::kisserror::{KissError,KissErrKind};
use fisco_bcos_rust_gears_sdk::bcossdk::bcossdk::BcosSDK;
use fisco_bcos_rust_gears_sdk::bcossdk::contractabi::ContractABI;
use fisco_bcos_rust_gears_sdk::bcossdk::contracthistory::ContractHistory;
use fisco_bcos_rust_gears_sdk::bcossdk::bcossdkquery;

use std::time::{Duration};
use std::thread;
use crate::{Cli, cmdmap};
use crate::{kisserr};
use serde_json::{Value as JsonValue};
use fisco_bcos_rust_gears_sdk::bcossdk::bcossdkquery::json_hextoint;
use fisco_bcos_rust_gears_sdk::bcossdk::bcosclientconfig::{ClientConfig,BcosCryptoKind};
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;
use fisco_bcos_rust_gears_sdk::bcossdk::solcompile::sol_compile;
use crate::console::cli_common::OptContract;
use crate::console::console_compile::console_compile;
use crate::console_cmdmap::CliCmdMap;
use crate::sample::demo_bcos3client::demo_bcos3client;


pub struct Bcos2Contract{
    pub cmdmap:CliCmdMap
}
impl Bcos2Contract {
    pub fn new() -> Self {
        let mut cmdhandler = Bcos2Contract {
            cmdmap: CliCmdMap::new("BCOS2 Contract")
        };
        //用宏定义实现将方法加到map里，宏里可以自动识别方法的名字，作为map的key
        //于是后续调用时，用名字去match就行了,且可以做到大小写不敏感
        cmdmap!(cmdhandler.cmdmap.cmd_func_map,   deploy);
        cmdmap!(cmdhandler.cmdmap.cmd_func_map,   call);
        cmdmap!(cmdhandler.cmdmap.cmd_func_map,   sendtx);
        cmdhandler
    }
}


pub fn deploy(cli:&Cli) ->Result<(),KissError>{

    let configfile=cli.default_configfile();
    let mut bcossdk = BcosSDK::new_from_config(configfile.as_str())?;
    println!("BcosSDK: {}",bcossdk.to_summary());
    //每次部署前强制编译一次对应合约，考虑到合约sol可能会有修改
    let res = console_compile(cli)?;

    let contractname =&cli.params[0];


    let params = &cli.params[1..];
    println!("deploy contract {} ,params:{:?}",contractname,params);

    let binfile = format!("{}/{}.bin",bcossdk.config.common.contractpath,contractname.to_string());
    let res = bcossdk.deploy_withparam(contractname.as_str(),&params);
    //println!("deploy transaction return :{:?}",&res);
    let txhash = match res{
        Ok(v)=>{let hash= String::from(v["result"].as_str().unwrap()); hash},
        Err(e)=>{return Err(e)}
    };
    let res = bcossdk.try_getTransactionReceipt(txhash.as_str(),3,false);
    match res {
        Ok(v)=>{
            let address = v["result"]["contractAddress"].as_str().unwrap();
            let blocknum = json_hextoint( &v["result"]["blockNumber"]).unwrap();
            println!("deploy contract on block[{}], address is {}",blocknum,address);
            let chf = ContractHistory::history_file(bcossdk.config.common.contractpath.as_str());
            let res = ContractHistory::save_to_file(chf.as_str(),"bcos2",contractname,address,blocknum as u64);
            println!("save contract history to file {} ,{:?}",chf,res);
        },
        Err(e)=>{
            return kisserr!(KissErrKind::ENetwork,"Deploy Fail {},{:?}",contractname,e);
        }
    }
    Ok(())
}
pub fn sendtx(cli:&Cli) ->Result<(),KissError>
{
    //将cmd和param拼在一起，作为新的args，给到StructOpt去解析（因为第一个参数总是app名）
    let mut cmdparams :Vec<String>= vec!(cli.cmd.clone());
    cmdparams.append(&mut cli.params.clone());
    let opt: OptContract = StructOpt::from_iter(cmdparams.iter());
    let configfile = cli.default_configfile();
    let mut bcossdk = BcosSDK::new_from_config(configfile.as_str())?;
    let contractdir = "contracts";
    let contractfullname = format!("{}/{}.abi",contractdir,&opt.contract_name);
    println!("contract file is {}",contractfullname);
    let contract = ContractABI::new(contractfullname.as_str(),&bcossdk.hashtype)?;
    let chfile = format!("{}/contracthistory.toml",bcossdk.config.common.contractpath);
    let address = ContractHistory::check_address_from_file(chfile.as_str(),"bcos2",
                                                           opt.contract_name.as_str(),opt.address.as_str())?;

    println!("contract address is {}",&address.as_str());
    let response = bcossdk.send_raw_transaction(&contract,address.as_str(),opt.method.as_str(),opt.params.as_slice())?;
    //println!("send_raw_transaction result {:?}", response);
    //println!("response[\"result\"] {:?}",response);
    let txhash = response["result"].as_str().unwrap();
    println!("\n>>>>>>>>>>>>>>>>>>>after sendtx getTransactionByHash");
    let receiptresult  = bcossdk.try_getTransactionReceipt(
        txhash,
        3,
        false);
    //println!("receiptresult result :{:?} ",receiptresult);
    match receiptresult {
        Ok(receipt) => {
            crate::console::console_utils::display_transaction_receipt(&receipt,&Option::from(&contract),&bcossdk.config);
        },
        Err(e) => {
            return kisserr!(KissErrKind::ENetwork,"{:?}",e)
        }
    };
    /*
    let txdata = bcossdk.getTransactionByHash(txhash).unwrap();
    let blocknum = bcossdkquery::json_hextoint(&txdata["result"]["blockNumber"]);
    let txinput = txdata["result"]["input"].as_str().unwrap();
    let inputdecode = contract.decode_input_for_tx(txinput).unwrap();
    println!("tx input :{:?}",inputdecode);*/
    Ok(())
}


pub fn call(cli:&Cli)->Result<(),KissError> {
    //将cmd和param拼在一起，作为新的args，给到StructOpt去解析（因为第一个参数总是app名）
    let mut cmdparams :Vec<String>= vec!(cli.cmd.clone());
    cmdparams.append(&mut cli.params.clone());
    let opt: OptContract = StructOpt::from_iter(cmdparams.iter());
    let configfile = cli.default_configfile();
    let mut bcossdk = BcosSDK::new_from_config(configfile.as_str())?;
    let contractdir = "contracts";
    let contractfullname = format!("{}/{}.abi", contractdir, &opt.contract_name);
    println!("contract file is {}", contractfullname);
    let contract = ContractABI::new(contractfullname.as_str(),&bcossdk.hashtype)?;
    let chfile = format!("{}/contracthistory.toml",bcossdk.config.common.contractpath);
    let address = ContractHistory::check_address_from_file(chfile.as_str(),"bcos2",
                                                           opt.contract_name.as_str(), opt.address.as_str())?;

    println!("contract address is {}", &address.as_str());
    let res = bcossdk.call(&contract, address.as_str(), opt.method.as_str(), opt.params.as_slice())?;
    println!("call result :{}",serde_json::to_string_pretty(&res).unwrap());
    let status = res["result"]["status"].as_str().unwrap();
    let ustatus = u32::from_str_radix(status.trim_start_matches("0x"), 16).unwrap();
    println!("call status code {} ({:?}) ", status, ustatus);
    if ustatus == 0
    {
        let output = res["result"]["output"].as_str().unwrap();
        let decodereuslt = contract.decode_output_byname(opt.method.as_str(), output);
        println!("call output: {:?}", decodereuslt);
    }else{
        return kisserr!(KissErrKind::Error,"call error !!!");
    }
    Ok(())
}



