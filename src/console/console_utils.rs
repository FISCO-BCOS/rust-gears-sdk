use ethabi::Log;
use serde::de::Unexpected::Option;

use serde_json::{Value as JsonValue, Value};

use crate::Cli;
use fisco_bcos_rust_gears_sdk::bcos2sdk::bcos2client::Bcos2Client;
use fisco_bcos_rust_gears_sdk::bcossdkutil::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use fisco_bcos_rust_gears_sdk::bcossdkutil::commonhash::CommonHash;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contractabi::ContractABI;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contracthistory::ContractHistory;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contracthistory::ContractRecord;
use fisco_bcos_rust_gears_sdk::bcossdkutil::kisserror::{KissErrKind, KissError};
use fisco_bcos_rust_gears_sdk::kisserr;

static DELOPY_ADDRESS: &str = "0000000000000000000000000000000000000000";

pub fn param_at(params: &Vec<String>, index: usize) -> Result<String, KissError> {
    if params.len() <= index as usize {
        return kisserr!(
            KissErrKind::EArgument,
            "params not enough,expect at least {}",
            index + 1
        );
    }
    return Ok(params[index].clone());
}
pub fn param_int(params: &Vec<String>, index: usize) -> Result<i32, KissError> {
    let v = param_at(params, index)?;

    if v.starts_with("0x") {
        match i32::from_str_radix(v.trim_start_matches("0x"), 16) {
            Ok(i) => {
                return Ok(i);
            }
            Err(e) => {
                return kisserr!(KissErrKind::EArgument, "param parse error {:?}", e);
            }
        }
    }

    match i32::from_str_radix(v.as_str(), 10) {
        Ok(i) => {
            return Ok(i);
        }
        Err(e) => {
            return kisserr!(KissErrKind::EArgument, "param parse error {:?}", e);
        }
    }
}

pub fn cli_groupid(cli: &Cli) -> u32 {
    return match param_int(&cli.params, 0) {
        Ok(i) => {
            return i as u32;
        }
        _ => cli.default_config().unwrap().bcos2.groupid,
    };
}

pub fn is_deploy_address(address: &str) -> bool {
    if address.trim_start_matches("0x").eq(DELOPY_ADDRESS) {
        return true;
    } else {
        return false;
    }
}
//如果没有prefix “0x”,就加上
pub fn align_address(address: &str) -> String {
    if !address.starts_with("0x") {
        let newaddress = format!("0x{}", address);
        return newaddress;
    }
    address.to_string()
}
///从历史里根据地址查找合约实例
pub fn find_contracthistory_byaddress(
    segment: &str,
    contractpath: &str,
    address_input: &str,
) -> Result<ContractRecord, KissError> {
    //从历史库中通过address获得contract对象
    let ch = ContractHistory::load_from_path(contractpath)?;
    //let address = align_address(address_input);
    let chrecord = ch.find_record_by_address(segment, address_input)?;
    //println!("find contract history by addess {:?}", chrecord);
    Ok(chrecord)
}

///根据名字或address去找一个合约实例
pub fn find_contract(
    segment: &str,
    contractname: &str,
    address: &str,
    config: &ClientConfig,
) -> Result<ContractABI, KissError> {
    if contractname.len() > 0 {
        println!("load contract by name : {}", contractname);
        let contract = ContractABI::new_by_name(
            contractname,
            config.common.contractpath.as_str(),
            &CommonHash::crypto_to_hashtype(&config.common.crypto),
        );
        return contract;
    } else {
        let chrecord =
            find_contracthistory_byaddress(segment, config.common.contractpath.as_str(), address)?;
        let splitres: Vec<&str> = chrecord.name.split("-chain").collect();
        let load_contract_name = splitres[0];
        let contract = ContractABI::new_by_name(
            load_contract_name,
            config.common.contractpath.as_str(),
            &CommonHash::crypto_to_hashtype(&config.common.crypto),
        );
        println!(
            "load contract {} by address : {}",
            chrecord.name, chrecord.address
        );
        return contract;
    }
}

///显示回执里的logs
pub fn display_receipt_logs(logs: &Vec<Log>) {
    let mut i = 0;
    for log in logs.iter() {
        i += 1;
        println!("{}) {:?})", i, log);
    }
}

///显示交易的回执,如果传入contract对象，则尝试解析input，output，logs，否则只打印基本信息
pub fn display_transaction_receipt(
    receipt_in: &JsonValue,
    contractopt: &std::option::Option<&ContractABI>,
    config: &ClientConfig,
) {
    let receipt_option = receipt_in.get("result");
    let receipt;
    match receipt_option {
        Some(v) => receipt = v.clone(),
        None => receipt = receipt_in.clone(),
    }
    println!("----------------------> Receipt summary-------------------->");
    //println!("receipt {}",serde_json::to_string_pretty(&receipt).unwrap());
    //output ,status,contractaddress,logs
    let blocknum: u64;
    let block_num_hex = receipt["blockNumber"].clone();
    if block_num_hex.is_string() {
        let blocknumstr = receipt["blockNumber"].as_str().unwrap();
        blocknum = u64::from_str_radix(blocknumstr.trim_start_matches("0x"), 16).unwrap();
    } else {
        blocknum = receipt["blockNumber"].as_u64().unwrap();
    }
    let istatus: i64;
    let statusvalue = receipt["status"].clone();
    if statusvalue.is_string() {
        let status: &str = receipt["status"].as_str().unwrap();
        istatus = i64::from_str_radix(status.trim_start_matches("0x"), 16).unwrap();
    } else {
        istatus = receipt["status"].as_i64().unwrap();
    }
    let contractaddress = receipt["contractAddress"].as_str().unwrap();
    let to = receipt["to"].as_str().unwrap();

    if is_deploy_address(to) {
        println!(
            "is Deploy Tx,on block [{}],new address :{} ",
            blocknum, contractaddress
        );
    } else {
        println!("is Normal Tx, on block [{}],to address :{} ", blocknum, to);
    }
    match contractopt {
        Some(contract) => {
            let outputstr = receipt["output"].as_str().unwrap();
            let inputstr = receipt["input"].as_str().unwrap();

            let inputres = contract.decode_input_for_tx(inputstr);

            match inputres {
                Ok(inputdetail) => {
                    let outputres = inputdetail.func.decode_output(outputstr.as_bytes());
                    let outputdetail = match outputres {
                        Ok(o) => o,
                        Err(e) => {
                            vec![]
                        } //置为空，供展示
                    };
                    let msg = format!(
                        "status:{},func:[{}],input: {:?},output:{:?}",
                        istatus, inputdetail.func.name, inputdetail.input, outputdetail
                    );
                    println!("{}", msg);
                }
                Err(e) => {
                    println!("decode input [{}] error {:?}", inputstr, e)
                } //置为空，供展示
            };

            let mut logs = receipt.get("logs");
            if logs == std::option::Option::None {
                logs = receipt.get("logEntries");
            }
            if logs != std::option::Option::None {
                let logs = contract.parse_receipt_logs(&logs.unwrap()).unwrap();
                display_receipt_logs(&logs);
            }
        }
        None => {}
    };
}

/// 判断是否部署合约
/// 如果调用合约，在历史表里找合约记录
/// 找到的额话，解析交易的input
/// 获取交易对应的receipt，打印receipt
pub fn display_transaction(
    v: &JsonValue,
    config: &ClientConfig,
    segment: &str,
    contractname: &str,
) -> Result<(), KissError> {
    println!("\n{}\n", serde_json::to_string_pretty(&v).unwrap());
    let to = v["to"].as_str().unwrap();
    println!("tx call to {}", to);
    if is_deploy_address(to) {
        //is deploy
        println!("a DEPLOY contract transaction");
        return Ok(());
    }

    let hash = v["hash"].as_str().unwrap();

    let contractres = find_contract(segment, contractname, to, &config);
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
    let input = contract.decode_input_for_tx(v["input"].as_str().unwrap());
    println!("Transction input is {:?}", input);

    Ok(())
}

/*
    let receipt = bcossdkutil.getTransactionReceipt(hash)?;
    println!("----------------------> Transaction summary-------------------->");
    let decordres = contract.decode_input_for_tx(v["result"]["input"].as_str().unwrap())?;
    println!("tx input : {:?}, params :{:?}", decordres.func.signature(), decordres.input);
*/
