use crate::bcossdk::kisserror::{KissError,KissErrKind};
use crate::bcossdk::bcossdk::BcosSDK;
use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::contracthistory::ContractHistory;
use crate::bcossdk::bcossdkquery;
use std::time::{Duration};
use std::thread;
use crate::{OptContract, Cli};
use crate::{kisserr};
use serde_json::{Value as JsonValue};
use crate::bcossdk::bcossdkquery::json_hextoint;

pub fn deploy(cli:&Cli) ->Result<(),KissError>{

    let configfile=match &cli.configfile{
        Some(s)=>{s.as_str()},
        None=>{"conf/config.toml"}
    };
    let mut bcossdk = BcosSDK::new_from_config(configfile)?;
    println!("BcosSDK: {}",bcossdk.to_summary());
    let contractname =&cli.params[0];
    let params = &cli.params[1..];
    println!("deploy contract {} ,params:{:?}",contractname,params);

    let binfile = format!("{}/{}.bin",bcossdk.config.contract.contractpath,contractname.to_string());
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
            let chf = ContractHistory::history_file(bcossdk.config.contract.contractpath.as_str());
            let res = ContractHistory::save_to_file(chf.as_str(),contractname,address,blocknum as u32);
            println!("save contract history to file {} ,{:?}",chf,res);
        },
        Err(e)=>{
            return kisserr!(KissErrKind::ENetwork,"Deploy Fail {},{:?}",contractname,e);
        }
    }
    Ok(())
}
pub fn sendtx(opt:&OptContract, configfile:&str) ->Result<(),KissError>
{
    let mut bcossdk = BcosSDK::new_from_config(configfile)?;
    let contractdir = "contracts";
    let contractfullname = format!("{}/{}.abi",contractdir,&opt.contract_name);
    println!("contract file is {}",contractfullname);
    let contract = ContractABI::new(contractfullname.as_str(),&bcossdk.hashtype)?;
    let chfile = format!("{}/contracthistory.toml",bcossdk.config.contract.contractpath);
    let address = ContractHistory::check_address_from_file(chfile.as_str(),
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


pub fn call(opt:&OptContract,configfile:&str)->Result<(),KissError> {
    let mut bcossdk = BcosSDK::new_from_config(configfile)?;
    let contractdir = "contracts";
    let contractfullname = format!("{}/{}.abi", contractdir, &opt.contract_name);
    println!("contract file is {}", contractfullname);
    let contract = ContractABI::new(contractfullname.as_str(),&bcossdk.hashtype)?;
    let chfile = format!("{}/contracthistory.toml",bcossdk.config.contract.contractpath);
    let address = ContractHistory::check_address_from_file(chfile.as_str(),
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
