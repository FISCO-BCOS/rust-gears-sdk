use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3client::Bcos3Client;
use fisco_bcos_rust_gears_sdk::bcossdkutil::commonhash::HashType;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contractabi::ContractABI;
use fisco_bcos_rust_gears_sdk::bcossdkutil::kisserror::KissError;
use fisco_bcos_rust_gears_sdk::bcossdkutil::liteutils::{datetime_str, json_str};

use crate::bcossdkutil::contracthistory::ContractHistory;
use crate::bcossdkutil::liteutils;
use crate::console::console_utils::display_transaction;
use crate::Cli;

pub fn demo_tx(cli: &Cli) -> Result<(), KissError> {
    let mut bcos3client = Bcos3Client::new(cli.default_configfile().as_str())?;
    // 1) deploy
    let result = bcos3client.deploy_file("contracts/HelloWorld.bin", "")?;
    println!("Deploy Result {:?}", result);
    let new_address = result["contractAddress"].as_str().unwrap();
    //1.1) save the address
    let chfile = format!(
        "{}/contracthistory.toml",
        bcos3client.config.common.contractpath
    );
    let blocknum = result["blockNumber"].as_u64().unwrap();
    //let bcos3contract_name = format!("HelloWorld-{}-{}", bcos3client.chainid, bcos3client.group);
    let res = ContractHistory::save_to_file(
        chfile.as_str(),
        bcos3client.get_full_name().as_str(),
        "HelloWorld",
        new_address,
        blocknum,
    );

    let contract_address = new_address; // "2237d46dada4c0306699555fc0bc6a31da29e4b4";
    let paramsvec = vec![format!("abcdefg : {}", datetime_str())];
    let contractabi = ContractABI::new_by_name(
        "HelloWorld",
        bcos3client.config.common.contractpath.as_str(),
        &bcos3client.hashtype,
    )?;
    // 2) send Transaction
    let methoname = "set";
    let result =
        bcos3client.sendTransaction(contract_address, methoname, &paramsvec, &contractabi)?;
    println!("send Transaction result {:?}", result);
    display_transaction(&result, &bcos3client.config, "bcos3", "HelloWorld")?;

    // 3) call
    let callresult = bcos3client.call(contract_address, "get", &vec![], &contractabi)?;
    println!("call result {:?}", callresult);
    let resultdata = callresult["output"].as_str().unwrap();
    let output = contractabi.decode_output_byname("get", resultdata).unwrap();
    println!("call output {:?}", output);
    bcos3client.finish();
    bcos3client.finish();
    Ok(())
}

pub fn demo_get(cli: &Cli) -> Result<(), KissError> {
    let mut bcos3client = Bcos3Client::new(cli.default_configfile().as_str())?;
    let blocknum = bcos3client.getBlockNumber()?;
    println!("getBlockNumber {:?}", blocknum);
    println!("getBlockLimit {:?}", bcos3client.getBlocklimit());
    println!("getPbfView {:?}", bcos3client.getPbftView());
    println!("getSealList {:?}", bcos3client.getSealerList());
    println!("getObserverList {:?}", bcos3client.getObserverList());
    println!("getConsensusStatus {:?}", bcos3client.getConsensusStatus());
    println!("getSyncStatus {:?}", bcos3client.getSyncStatus());
    println!("getPeers {:?}", bcos3client.getPeers());
    println!("getGroupPeers {:?}", bcos3client.getGroupPeers());
    println!("getGroupList {:?}", bcos3client.getGroupList());
    let block = bcos3client.getBlockByNumber(blocknum, 0, 1).unwrap();
    println!("getBlockByNumber {:?}", block);
    println!("block detail {:?}", block);
    println!(
        "blocknum {},blockhash {}",
        liteutils::json_u64(&block, "number", -1),
        liteutils::json_str(&block, "hash", "")
    );
    let hash = bcos3client.getBlockHashByNumber(blocknum).unwrap();
    println!("getBlockHashByNumber {}, {:?}", blocknum, &hash);
    let block = bcos3client.getBlockByHash(hash.as_str(), 0, 1)?;
    println!("getBlockByHash {:?}", block);
    println!(
        "blocknum {},blockhash {}",
        liteutils::json_u64(&block, "number", -1),
        liteutils::json_str(&block, "hash", "")
    );

    println!(
        "getTotalTransactionCount {:?}",
        bcos3client.getTotalTransactionCount()
    );
    let txlist = block["transactions"].as_array().unwrap().clone();
    let value = txlist.get(0).unwrap().clone();
    let txhash = value.as_str().unwrap().clone();
    println!("get transaction by hash : {}", &txhash);
    let tx = bcos3client.getTransactionByHash(txhash, 0)?;
    println!("get transction by hash result {:?}", tx);
    let receipt = bcos3client.getTransactionReceipt(txhash, 0);
    println!("get receipt  result {:?}", receipt);
    println!("getPendingTxSize {:?}", bcos3client.getPendingTxSize());
    //println!("getPeers {:?}",bcos3client.getPeers());
    bcos3client.finish();
    Ok(())
}

pub fn demo_bcos3client(cli: Cli) -> Result<(), KissError> {
    println!("--->>>user input {:?}", cli);
    match cli.params[0].as_str() {
        "tx" => {
            demo_tx(&cli)?;
        }
        "get" => {
            demo_get(&cli)?;
        }
        _ => {
            println!("");
        }
    }
    Ok(())
}
