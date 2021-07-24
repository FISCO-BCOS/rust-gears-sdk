use ethabi::Log;
use serde_json::Value as JsonValue;

use crate::bcossdk::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use crate::bcossdk::bcossdk::BcosSDK;
use crate::bcossdk::commonhash::CommonHash;
use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::contracthistory::ContractHistory;
use crate::bcossdk::kisserror::KissErrKind;
use crate::bcossdk::kisserror::KissError;
use crate::Cli;
use crate::kisserr;

static DELOPY_ADDRESS: &str = "0000000000000000000000000000000000000000";


pub fn param_at(params: &Vec<String>, index: usize) -> Result<String, KissError>
{
    if params.len() <= index as usize
    {
        return kisserr!(KissErrKind::EArgument,"params not enough,expect at least {}",index+1);
    }
    return Ok(params[index].clone())
}
pub fn param_int(params: &Vec<String>, index: usize) -> Result<i32, KissError>
{
    let v= param_at(params,index)?;

    if v.starts_with("0x"){
        match i32::from_str_radix(v.trim_start_matches("0x"), 16) {
            Ok(i) => { return Ok(i); }
            Err(e) => { return kisserr!(KissErrKind::EArgument,"param parse error {:?}",e); }
        }
    }

    match i32::from_str_radix(v.as_str(), 10) {
        Ok(i) => { return Ok(i); }
        Err(e) => { return kisserr!(KissErrKind::EArgument,"param parse error {:?}",e); }
    }
}

pub fn cli_groupid(cli: &Cli) -> u32
{
    return match param_int(&cli.params, 0) {
        Ok(i) => { return i as u32; }
        _ => {
            cli.default_config().unwrap().chain.groupid
        }
    };
}


pub fn is_deploy_address(address: &str) -> bool {
    if address.trim_start_matches("0x").eq(DELOPY_ADDRESS) {
        return true;
    } else {
        return false;
    }
}

///从历史里根据地址查找合约实例
pub fn find_contracthistory_byaddress(contractpath: &str, crytokind: &BcosCryptoKind, address: &str) -> Result<ContractABI, KissError>
{
    //从历史库中通过address获得contract对象
    let ch = ContractHistory::load_from_path(contractpath)?;
    let chrecord = ch.find_record_by_address(address)?;
    println!("find contract history by addess {:?}", chrecord);
    let contract = ContractABI::new_by_name(chrecord.name.as_str(),
                                            contractpath,
                                            &CommonHash::crypto_to_hashtype(crytokind))?;
    Ok(contract)
}


///根据名字或address去找一个合约实例
pub fn find_contract(nameopt: &Option<String>, address: &str, config: &ClientConfig) -> Result<ContractABI, KissError>
{
    let contract = match &nameopt {
        Some(name) => {
            let contract = ContractABI::new_by_name(name,
                                                    config.contract.contractpath.as_str(),
                                                    &CommonHash::crypto_to_hashtype(&config.chain.crypto),
            );
            println!("load contract by name : {}", name);
            return contract;
        }
        Option::None => {
            return find_contracthistory_byaddress(config.contract.contractpath.as_str(),
                                                  &config.chain.crypto,
                                                  address);
        }
    };
}


///显示回执里的logs
pub fn display_receipt_logs(logs: &Vec<Log>)
{
    let mut i = 0;
    for log in logs.iter() {
        i += 1;
        println!("{}) {:?})", i, log);
    }
}

///显示交易的回执,如果传入contract对象，则尝试解析input，output，logs，否则只打印基本信息
pub fn display_transaction_receipt(receipt: &JsonValue, contractopt: &Option<&ContractABI>, config: &ClientConfig)
{
    println!("----------------------> Receipt summary-------------------->");
    //output ,status,contractaddress,logs
    let blocknumstr = receipt["result"]["blockNumber"].as_str().unwrap();
    let blocknum = u32::from_str_radix(blocknumstr.trim_start_matches("0x"),16).unwrap();
    let status: &str = receipt["result"]["status"].as_str().unwrap();
    let contractaddress = receipt["result"]["contractAddress"].as_str().unwrap();
    let to = receipt["result"]["to"].as_str().unwrap();
    let istatus = i32::from_str_radix(status.trim_start_matches("0x"), 16).unwrap();
    if is_deploy_address(to) {
        println!("is Deploy Tx,on block [{}],new address :{} ",blocknum, contractaddress);
    } else {
        println!("is Normal Tx, on block [{}],to address :{} ",blocknum, to);
    }
    match contractopt {
        Some(contract) => {
            let outputstr = receipt["result"]["output"].as_str().unwrap();
            let inputstr = receipt["result"]["input"].as_str().unwrap();

            let inputres = contract.decode_input_for_tx(inputstr);

            match inputres {
                Ok(inputdetail) => {
                    let outputres = inputdetail.func.decode_output(outputstr.as_bytes());
                    let outputdetail = match outputres {
                        Ok(o) => { o }
                        Err(e) => { vec!() } //置为空，供展示
                    };
                    let msg = format!("status:{},func:[{}],input: {:?},output:{:?}",
                                      istatus, inputdetail.func.name, inputdetail.input, outputdetail);
                    println!("{}", msg);
                }
                Err(e) => { println!("decode input [{}] error {:?}",inputstr,e) } //置为空，供展示
            };

            let logs = contract.parse_receipt_logs(&receipt["result"]["logs"]).unwrap();
            display_receipt_logs(&logs);
        }
        None => {}
    };
}


/// 判断是否部署合约
/// 如果调用合约，在历史表里找合约记录
/// 找到的额话，解析交易的input
/// 获取交易对应的receipt，打印receipt
pub fn display_transaction(v: &JsonValue, bcossdk: &mut BcosSDK, cli: &Cli) -> Result<(), KissError>
{
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    let to = v["result"]["to"].as_str().unwrap();
    println!("tx call to {}", to);
    if is_deploy_address(to) { //is deploy
        println!("a DEPLOY contract transaction");
        return Ok(());
    }

    let hash = v["result"]["hash"].as_str().unwrap();

    let contractres = find_contract(&cli.contractname, to, &bcossdk.config);
    let contract = match contractres {
        Ok(c) => { c }
        Err(e) => {
            //miss ,but maybe not a error
            println!("Missing contract history for address  [{}] ,but ok &done ", to);
            return Ok(());
        }
    };

    let receipt = bcossdk.getTransactionReceipt(hash)?;
    println!("----------------------> Transaction summary-------------------->");
    let decordres = contract.decode_input_for_tx(v["result"]["input"].as_str().unwrap())?;
    println!("tx input : {:?}, params :{:?}", decordres.func.signature(), decordres.input);
    Ok(())
}
