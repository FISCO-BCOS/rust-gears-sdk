/*---------------------------------
https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html
----------------------------------*/
/*
  FISCO BCOS/rust-SDK is a rust client for FISCO BCOS2.0 (https://github.com/FISCO-BCOS/)
  FISCO BCOS/rust-SDK is free software: you can redistribute it and/or modify it under the
  terms of the MIT License as published by the Free Software Foundation. This project is
  distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
  the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
  @author: kentzhang
  @date: 2021-07
*/
#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_variables,
    unused_assignments
)]

use ethabi::Log as ReceiptLog;
use serde_json::{json, Value as JsonValue};

use crate::bcossdk::bcossdk::BcosSDK;
use crate::bcossdk::commonhash::HashType;
use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::kisserror::{KissErrKind, KissError};
use std::thread;
use std::time::Duration;

const DELTABLOCKLIMIT: u32 = 500;

/*从json中获得类似0x123这样的hex值，转成int*/
pub fn json_hextoint(v: &JsonValue) -> Result<i32, KissError> {
    let num_option = v.as_str();
    match num_option {
        Some(v) => {
            let s = v.trim_start_matches("0x");
            let numres = i32::from_str_radix(s, 16);
            match numres {
                Ok(num) => Ok(num),
                Err(e) => {
                    kisserr!(KissErrKind::EFormat, "result format error {:?}", v)
                }
            }
        }
        _ => {
            return kisserr!(KissErrKind::EFormat, "result format error {:?}", v);
        }
    }
}

///--------------在这个文件里聚合一下查询接口,用简单的公共方法提供----------------------------------------
///方法命令刻意和https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html
///保持大小写和拼写一致，以便查找，局部不遵循rust命令规范
impl BcosSDK {
    pub fn getBlockNumber(&mut self) -> Result<u32, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getBlockNumber";
        let paramobj = json!([groupid]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        let num = json_hextoint(&v["result"])? as u32;
        Ok(num)
    }

    /*
        refer to Python SDK: https://github.com/FISCO-BCOS/python-sdk
    */
    pub fn getBlockLimit(&mut self) -> Result<u32, KissError> {
        let now = time::now();
        printlnex!("getblocknumber cause :{}", now - self.updateblocknum_tick);
        //每30秒获取一次
        if now - self.updateblocknum_tick > time::Duration::seconds(30) {
            let block_num = self.getBlockNumber()?;
            self.lastblocknum = block_num;
            self.updateblocknum_tick = time::now();
        }

        Ok(self.lastblocknum + DELTABLOCKLIMIT)
    }

    pub fn getNodeVersion(&mut self) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getClientVersion";
        let paramobj = json!([groupid]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(v["result"].clone())
    }

    pub fn getNodeInfo(&mut self) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getNodeInfo";
        let paramobj = json!([groupid]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(v["result"].clone())
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getpbftview
    /// curl -X POST --data '{"jsonrpc":"2.0","method":"getPbftView","params":[1],"id":1}' http://127.0.0.1:8545 |jq
    ///
    /// Result
    ///{
    ///    "id": 1,
    ///    "jsonrpc": "2.0",
    ///    "result": "0x1a0"
    ///}
    pub fn getPbftView(&mut self, groupid: u32) -> Result<i32, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getPbftView";
        let paramobj = json!([groupid]);
        let res = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        json_hextoint(&res["result"])
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getsealerlist
    /// curl -X POST --data '{"jsonrpc":"2.0","method":"getSealerList","params":[1],"id":1}' http://127.0.0.1:8545 |jq
    //
    // // Result
    // {
    //     "id": 1,
    //     "jsonrpc": "2.0",
    //     "result": [
    //         "037c255c06161711b6234b8c0960a6979ef039374ccc8b723afea2107cba3432dbbc837a714b7da20111f74d5a24e91925c773a72158fa066f586055379a1772",
    //         "0c0bbd25152d40969d3d3cee3431fa28287e07cff2330df3258782d3008b876d146ddab97eab42796495bfbb281591febc2a0069dcc7dfe88c8831801c5b5801",
    //         "622af37b2bd29c60ae8f15d467b67c0a7fe5eb3e5c63fdc27a0ee8066707a25afa3aa0eb5a3b802d3a8e5e26de9d5af33806664554241a3de9385d3b448bcd73"
    //     ]
    // }
    pub fn getSealerList(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getSealerList";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getobserverlist
    pub fn getObserverList(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getObserverList";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getconsensusstatus
    pub fn getConsensusStatus(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getConsensusStatus";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getsyncstatus
    pub fn getSyncStatus(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getSyncStatus";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getpeers
    pub fn getPeers(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getPeers";
        let paramobj = json!([self.config.bcos2.groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getgrouppeers
    pub fn getGroupPeers(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getGroupPeers";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getnodeidlist
    pub fn getNodeIDList(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getNodeIDList";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getgrouplist
    pub fn getGroupList(&mut self) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getGroupList";
        let paramobj = json!([self.config.bcos2.groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getblockbynumber
    pub fn getBlockByNumber(
        &mut self,
        num: u32,
        includeTransactions: bool,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getBlockByNumber";
        let hexnum = format!("0x{:02X}", num);
        let paramobj = json!([self.config.bcos2.groupid, hexnum, includeTransactions]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getBlockHashByNumber
    pub fn getBlockHashByNumber(&mut self, num: u32) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getBlockHashByNumber";
        let hexnum = format!("0x{:02X}", num);
        let paramobj = json!([self.config.bcos2.groupid, hexnum.to_string()]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }
    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getblockbynumber
    pub fn getBlockHeaderByNumber(
        &mut self,
        num: u32,
        includeTransactions: bool,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getBlockHeaderByNumber";
        let hexnum = format!("0x{:02X}", num);
        let paramobj = json!([
            self.config.bcos2.groupid,
            hexnum.to_string(),
            includeTransactions
        ]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getblockbyhash
    pub fn getBlockByHash(
        &mut self,
        blockhash: &str,
        includeTransactions: bool,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getBlockByHash";
        let paramobj = json!([
            self.config.bcos2.groupid,
            blockhash.to_string(),
            includeTransactions
        ]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getblockheaderbyhash
    pub fn getBlockHeaderByHash(
        &mut self,
        blockhash: &str,
        includeTransactions: bool,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getBlockHeaderByHash";
        let paramobj = json!([
            self.config.bcos2.groupid,
            blockhash.to_string(),
            includeTransactions
        ]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }


    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#gettransactionreceipt
    pub fn getTransactionReceipt(&mut self, txhash: &str) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getTransactionReceipt";
        let paramobj = json!([groupid, txhash]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    pub fn try_getTransactionReceipt(
        &mut self,
        txhash: &str,
        timeoutsec: i64,
        allow_none_result: bool,
    ) -> Result<JsonValue, KissError> {
        let start = time::now();
        //let h = "0xd47832f4de959582fc1964cea04da09506200c41a81e59c8934b23017deca27a";
        while time::now() - start < chrono::Duration::seconds(timeoutsec) {
            //println!("go get receipt");
            let v = self.getTransactionReceipt(txhash)?;
            //println!("result {:?}",v);
            if v["result"] == JsonValue::Null {
                if allow_none_result {
                    return Ok(v);
                }
                thread::sleep(Duration::from_millis(200));
                continue;
            }
            return Ok(v);
        }
        return kisserr!(
            KissErrKind::ENetwork,
            "getTransactionReceipt timeout or missing"
        );
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#gettransactionreceiptbyhashwithproof
    pub fn getTransactionReceiptByHashWithProof(
        &mut self,
        txhash: &str,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getTransactionReceiptByHashWithProof";
        let paramobj = json!([groupid, txhash]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#gettransactionreceipt
    /// 调用getTransactionReceipt,且按传入的abi,解析出log,receipt的其他字段不会返回。常用于demo
    /// 所以需要receipt其他字段的话，直接调用getTransactionReceipt再解析
    pub fn getTransactionReceiptlogs(
        &mut self,
        txhash: &str,
        contract: &ContractABI,
    ) -> Result<Vec<ReceiptLog>, KissError> {
        let receipt = self.getTransactionReceipt(txhash)?;
        contract.parse_receipt_logs(&receipt["result"]["logs"])
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#gettransactionbyhash
    pub fn getTransactionByHash(&mut self, txhash: &str) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getTransactionByHash";
        let paramobj = json!([groupid, txhash]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#gettransactionbyblockhashandindex
    pub fn getTransactionByBlockHashAndIndex(
        &mut self,
        blockhash: &str,
        index: u32,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getTransactionByBlockHashAndIndex";
        let indexhex = format!("0x{:02X}", index);
        let paramobj = json!([groupid, blockhash, indexhex]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }
    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#gettransactionbyblocknumberandindex
    pub fn getTransactionByBlockNumberAndIndex(
        &mut self,
        blocknum: u32,
        index: u32,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getTransactionByBlockNumberAndIndex";
        let blocknum = format!("0x{:02X}", blocknum);
        let indexhex = format!("0x{:02X}", index);
        let paramobj = json!([groupid, blocknum, indexhex]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#gettransactionbyhashwithproof
    pub fn getTransactionByHashWithProof(&mut self, txhash: &str) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getTransactionByHashWithProof";
        let paramobj = json!([groupid, txhash]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getpendingtransactions
    pub fn getPendingTransactions(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getPendingTransactions";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    pub fn getPendingTxSize(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getPendingTxSize";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    pub fn getTotalTransactionCount(&mut self, groupid: u32) -> Result<JsonValue, KissError> {
        let cmd = "getTotalTransactionCount";
        let paramobj = json!([groupid]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getcode
    pub fn getCode(&mut self, groupid: u32, address: &str) -> Result<JsonValue, KissError> {
        let cmd = "getCode";
        let paramobj = json!([groupid, address.to_string()]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getsystemconfigbykey
    pub fn getSystemConfigByKey(
        &mut self,
        groupid: u32,
        key: &str,
    ) -> Result<JsonValue, KissError> {
        let cmd = "getSystemConfigByKey";
        let paramobj = json!([groupid, key.to_string()]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }



    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getbatchreceiptsbyblocknumberandrange
    /// curl -X POST --data '{"jsonrpc":"2.0","method":"getBatchReceiptsByBlockNumberAndRange","params":[1,"0x1","0","-1",false],"id":1}' http://127.0.0.1:8545 |jq
    // {
    //   "id": 1,
    //   "jsonrpc": "2.0",
    //   "result": {
    //     "blockInfo": {
    //       "blockHash": "0xcef82a3c1e7770aa4e388af5c70e97ae177a3617c5020ae052be4095dfdd39a2",
    //       "blockNumber": "0x1",
    //       "receiptRoot": "0x69a04fa6073e4fc0947bac7ee6990e788d1e2c5ec0fe6c2436d0892e7f3c09d2",
    //       "receiptsCount": "0x1"
    //     },
    //     "transactionReceipts": [
    //       {
    //         "contractAddress": "0x0000000000000000000000000000000000000000",
    //         "from": "0xb8e3901e6f5f842499fd537a7ac7151e546863ad",
    //         "gasUsed": "0x5798",
    //         "logs": [],
    //         "output": "0x08c379a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000364572726f7220616464726573733a3863313763663331366331303633616236633839646638373565393663396630663562326637343400000000000000000000",
    //         "status": "0x1a",
    //         "to": "0x8c17cf316c1063ab6c89df875e96c9f0f5b2f744",
    //         "transactionHash": "0xc6ec15fd1e4d696e66d7fbef6064bda3ed012bcb7744d09903ee289df65f7d53",
    //         "transactionIndex": "0x0"
    //       }
    //     ]
    //   }
    // }
    ///groupID: 群组ID;
    ///blockNumber: 请求获取的回执信息所在的区块高度;
    ///from: 需要获取的回执起始索引;
    ///count: 需要批量获取的回执数目，当设置为-1时，返回区块内所有回执信息;
    ///compressFlag: 压缩标志，当设置为false时，以明文的形式返回批量交易回执信息; 当设置为true时, 以zlib格式压缩批量交易回执，并将压缩后的回执信息以Base64编码的格式返回。
    pub fn getBatchReceiptsByBlockNumberAndRange(
        &mut self,
        blockNumber: u32,
        from: u32,
        count: i32,
        compressFlag: bool,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getBatchReceiptsByBlockNumberAndRange";
        //params":[1,"0x1","0","-1",false]
        let blocknumhex = format!("0x{:X}", blockNumber);
        let paramobj = json!([
            groupid,
            blocknumhex,
            format!("{}", from),
            format!("{}", count),
            compressFlag
        ]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }
    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#getbatchreceiptsbyblockhashandrange
    pub fn getBatchReceiptsByBlockHashAndRange(
        &mut self,
        blockhash: &str,
        from: u32,
        count: i32,
        compressFlag: bool,
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.bcos2.groupid;
        let cmd = "getBatchReceiptsByBlockHashAndRange";
        // curl -X POST --data '{"jsonrpc":"2.0","method":"getBatchReceiptsByBlockHashAndRange","params":[1,"0xcef82a3c1e7770aa4e388af5c70e97ae177a3617c5020ae052be4095dfdd39a2","0","1",false],"id":1}' http://127.0.0.1:8545 |jq
        let paramobj = json!([
            groupid,
            blockhash,
            format!("{}", from),
            format!("{}", count),
            compressFlag
        ]);
        self.netclient.rpc_request_sync(cmd, &paramobj)
    }
}

pub fn demo_query() {
    let mut bcossdk = BcosSDK::new().unwrap();
    let res = bcossdk.getBlockNumber();
    println!("getBlockNumber {:?}", res);
    println!("getNodeVersion {:?}", bcossdk.getNodeVersion()); //serde_json::to_string_pretty(&res).unwrap());
    println!("getNodeInfo {:?}", bcossdk.getNodeInfo());

    let txhash = "0x28c4717ce415f3e3b4e69b9b38f205cb0f73fb40c4faf85836af9083a7457a6c";
    let res = bcossdk.getTransactionByHash(txhash).unwrap();
    println!("getTransactionByHash {:?}", res); // serde_json::to_string_pretty(&res).unwrap());
    let contract = ContractABI::new("contracts/HelloWorld.abi", &HashType::WEDPR_KECCAK).unwrap();
    let decoderes = contract.decode_input_for_tx(res["result"]["input"].as_str().unwrap());
    println!("decode reuslt : {:?}", decoderes);

    println!("getPbftView : {:?}", bcossdk.getPbftView(1));
    println!("getSealerList {:?}", bcossdk.getSealerList(1));
    println!("getConsensusStatus {:?}", bcossdk.getConsensusStatus(1));

    //block serial
    let res = bcossdk.getBlockByNumber(1, true).unwrap();
    println!("getBlockByNumber res:{:?}", &res); //serde_json::to_string_pretty(&(res.unwrap())).unwrap());
    let blockhash = res["result"]["hash"].as_str().unwrap();
    println!("blockhash is  {}", blockhash);
    let res = bcossdk.getBlockByHash(blockhash, false);
    println!("getBlockByHash {:?}", res);
    let res = bcossdk.getBlockHeaderByNumber(10, true).unwrap();
    println!("getBlockHeaderByNumber {:?}", res);
    let blockhash = res["result"]["hash"].as_str().unwrap();

    let res = bcossdk.getBlockHeaderByHash(blockhash, true);
    println!("getBlockHeaderByHash {:?}", res);

    let res = bcossdk.getTransactionByBlockHashAndIndex(blockhash, 0);
    println!("getTransactionByBlockHashAndIndex {:?}", res);

    let res = bcossdk.getTransactionByBlockNumberAndIndex(2, 0);
    println!("getTransactionByBlockNumberAndIndex {:?}", res);
    let txresult = res.unwrap();
    let txhash = txresult["result"]["hash"].as_str().unwrap();
    let res = bcossdk.getTransactionByHashWithProof(txhash);
    println!("getTransactionByHashWithProof {:?}", res);

    println!(
        "getPendingTransactions {:?}",
        bcossdk.getPendingTransactions(1)
    );
    println!("getPendingTxSize {:?}", bcossdk.getPendingTxSize(1));
    let totalres = bcossdk.getTotalTransactionCount(1).unwrap();
    println!("getTotalTransactionCount {:?}", &totalres);
    println!(
        "blocknum :{:?},failtx {:?}, txsum :{:?}",
        json_hextoint(&totalres["result"]["blockNumber"]),
        json_hextoint(&totalres["result"]["failedTxSum"]),
        json_hextoint(&totalres["result"]["txSum"]),
    );
    let res = bcossdk.getBatchReceiptsByBlockNumberAndRange(2, 0, -1, false);
    println!("getBatchReceiptsByBlockNumberAndRange {:?}", res);
    let detail = res.unwrap();
    let blockhash = detail["result"]["blockInfo"]["blockHash"].as_str().unwrap();

    let res = bcossdk.getBatchReceiptsByBlockHashAndRange(blockhash, 0, -1, false);
    println!("getBatchReceiptsByBlockHashAndRange {:?}", res);

    println!("{:?}", bcossdk.getBlockNumber());
    bcossdk.finish();
}
