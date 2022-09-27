#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_imports,
    unused_results,
    unused_variables
)]
use std::time::Duration;
use fisco_bcos_rust_gears_sdk::bcossdk::bcossdk::BcosSDK;
use fisco_bcos_rust_gears_sdk::bcossdk::contractabi::ContractABI;
use fisco_bcos_rust_gears_sdk::bcossdk::kisserror::KissError;
use fisco_bcos_rust_gears_sdk::bcossdk::{bcossdkquery, fileutils};
use std::thread;
use serde_json::{json, Value as JsonValue};
use fisco_bcos_rust_gears_sdk::bcossdk::solcompile::sol_compile;
use fisco_bcos_rust_gears_sdk::bcossdk::contracthistory::ContractHistory;
use fisco_bcos_rust_gears_sdk::bcossdk::bcossdkquery::json_hextoint;
use crate::Cli;
use crate::console::console_utils::display_transaction_receipt;

pub fn demo_deploy(bcossdk: &mut BcosSDK, contract:&ContractABI) -> Result<String,KissError>
{
    let contract_name = "ArrayDemo";
    let compileres  = sol_compile(contract_name,&bcossdk.config.configfile.as_ref().unwrap().as_str());
    println!("compile result:{:?}",compileres);
    let params:[String;2]=["default text 009".to_string(),"199".to_string()];

    let binfile = format!("{}/{}.bin",bcossdk.config.common.contractpath,contract_name.to_string());
    let v = bcossdk.deploy_file(binfile.as_str(), "");
    println!("request response {:?}", v);
    let response = v.unwrap();
    let txhash = response["result"].as_str().unwrap();
    let recepitresult = bcossdk.try_getTransactionReceipt( txhash,3,false);
    println!("receipt {:?}",recepitresult);
    let receipt = recepitresult.unwrap();
    let addr:String = receipt["result"]["contractAddress"].as_str().unwrap().to_string();
    let blocknum = json_hextoint(&receipt["result"]["blockNumber"]).unwrap();
    println!("deploy contract on block {}",blocknum);
    let history_file = ContractHistory::history_file(bcossdk.config.common.contractpath.as_str());
    let res = ContractHistory::save_to_file(history_file.as_str(),"bcos2","ArrayDemo",addr.as_str(),blocknum as u64);

    Ok(addr)
}



//---------------------------------------------------------
pub fn demo(cli:&Cli)
{
    let contract_name = "ArrayDemo";
    let configfile = cli.default_configfile();
    let mut bcossdk = BcosSDK::new_from_config(configfile.as_str()).unwrap();

    let contract = ContractABI::new_by_name(contract_name,
                                            bcossdk.config.common.contractpath.as_str(),
                                            &bcossdk.hashtype).unwrap();
    println!("\n>>>>>>>>>>>>>>>>>>demo deploy contract");
    let newaddr = demo_deploy(&mut bcossdk,&contract).unwrap();
    println!("new addr {}",&newaddr);
    let to_address = newaddr;
    println!(">>>>>>>>>>>>>>>> sendtx add transaction");
    let mut params:Vec<String> = vec!("99".to_string());
    let data:Vec<String> = vec!("aa".to_string(),"bb".to_string(),"cc".to_string());
    let datastr =ContractABI::array_to_param(&data);
    println!("datastr :--> {} <---",datastr);
    params.push(datastr);
    println!("all params :{:?}",params);

    let res = bcossdk.sendRawTransactionGetReceipt(&contract,&to_address,"add",params.as_slice()).unwrap();
    println!("send transaction result {:?}",res);
    display_transaction_receipt(&res,&Option::from(&contract),&bcossdk.config);

    println!(">>>>>>>>>>>>>>>> sendtx add transaction next time");
    let mut params:Vec<String> = vec!("9999".to_string());
    let data:Vec<String> = vec!("beijing".to_string(),"shenzhen".to_string(),"shanghai".to_string(),"guangzhou".to_string());
    let datastr =ContractABI::array_to_param(&data);
        println!("datastr :--> {} <---",&datastr);
    params.push(datastr);
    println!("all params :{:?}",params);
    let res = bcossdk.sendRawTransactionGetReceipt(&contract,&to_address,"add",params.as_slice()).unwrap();
    println!("send transaction result {:?}",res);
    display_transaction_receipt(&res,&Option::from(&contract),&bcossdk.config);


    println!(">>>>>>>>>>>>>>>> call after transaction");
    let callvalue = bcossdk.call(&contract, &to_address, "total", &["".to_string()]).unwrap();
    let output = callvalue["result"]["output"].as_str().unwrap();

    let decodereuslt = contract.decode_output_byname("total", output).unwrap();
    println!("total function output: {:?}",decodereuslt);
    let totalToken = decodereuslt.get(0);
    let total = totalToken.unwrap();

    let index = total.clone().to_uint().unwrap()-2;
    println!("get index = {}",index);
    let callvalue = bcossdk.call(&contract, &to_address, "get", &[index.to_string()]).unwrap();
    let output = callvalue["result"]["output"].as_str().unwrap();

    let decodereuslt = contract.decode_output_byname("get", output);
    println!("get function output: {:?}",decodereuslt);

    println!("demo on : {:?}",bcossdk.getNodeVersion());
    bcossdk.finish();
}
