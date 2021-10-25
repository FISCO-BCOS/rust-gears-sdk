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

use ethereum_types::U256;
use serde_json::{json, Value as JsonValue};
use time::Tm;
use log::info;
use crate::bcossdk::accountutil::{account_from_pem, BcosAccount};
use crate::bcossdk::bcosclientconfig::BcosClientProtocol;
use crate::bcossdk::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use crate::bcossdk::bcosrpcwraper::BcosRPC;
use crate::bcossdk::bcostransaction::{BcosTransaction, BcosTransactionWithSig};
use crate::bcossdk::commonhash::{CommonHash, HashType};
use crate::bcossdk::commonsigner::{
    CommonSignerWeDPR_SM2, CommonSignerWeDPR_Secp256, ICommonSigner,
};
use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::fileutils;
use crate::bcossdk::kisserror::{KissErrKind, KissError};
use std::process::{Command, Output};
use std::path::{PathBuf, Path};
use ethabi::Token;

#[derive()]
pub struct BcosSDK {
    pub config: ClientConfig,
    pub account: BcosAccount,
    pub netclient: BcosRPC,
    pub ecdsasigner: Option<CommonSignerWeDPR_Secp256>,
    pub gmsigner: Option<CommonSignerWeDPR_SM2>,
    //重要：当前sdk实例采用的hash算法，如keccak,国密等，当前客户端的编解码，签名都必须基于相同的hash算法
    //主要牵涉： account生成和加载，transaction签名，abi编解码
    pub hashtype: HashType,
    pub updateblocknum_tick: Tm,
    pub lastblocknum: u32,
}
//unsafe impl Send for BcosSDK{}
//unsafe impl Sync for BcosSDK{}

impl BcosSDK {
    pub fn to_summary(&self) -> String {
        let basic = format!(
            "Crypto kind:{:?},configfile:{}",
            &self.config.chain.crypto,
            &self.config.configfile.as_ref().unwrap()
        );
        let protocol;
        match self.config.chain.protocol {
            BcosClientProtocol::RPC => {
                protocol = format!("RPC:{}", self.config.rpc.url);
            }
            BcosClientProtocol::CHANNEL => {
                protocol = format!(
                    "Channel:{}:{}({:?})",
                    &self.config.channel.ip, self.config.channel.port, &self.config.channel.tlskind
                );
            }
        }
        let full = format!("{},{}", protocol, basic);
        return full;
    }

    pub fn new_from_config(configfile: &str) -> Result<BcosSDK, KissError> {
        let config = ClientConfig::load(configfile)?;
        printlnex!("config is {:?}", config);

        //国密和非国密关键步骤，设置hash,账户类型和签名的密码学方法
        //一定要先设置hash算法，这是基础中的基础
        let hashtype = CommonHash::crypto_to_hashtype(&config.chain.crypto);

        let mut ecdsasigner = Option::None;
        let mut gmsigner = Option::None;
        let account = account_from_pem(config.chain.accountpem.as_str(), &config.chain.crypto)?;
        //printlnex!("done account");
        match &config.chain.crypto {
            BcosCryptoKind::ECDSA => {
                let mut signer = CommonSignerWeDPR_Secp256::default();
                signer.account = account.clone();
                ecdsasigner = Option::from(signer);
            }
            BcosCryptoKind::GM => {
                let mut signer = CommonSignerWeDPR_SM2::default();
                signer.account = account.clone();
                gmsigner = Option::from(signer);

            }
        }
        let netclient = BcosRPC::new(&config)?;

        Ok(BcosSDK {
            config,
            account,
            netclient,
            gmsigner: gmsigner.clone(),
            ecdsasigner: ecdsasigner.clone(),
            hashtype: hashtype.clone(),
            //默认初始化为500秒前，以便第一次一定会去取一下blocknum
            updateblocknum_tick: time::now() - chrono::Duration::seconds(500),
            lastblocknum: 0,
        })
    }

    ///加载配置并创建一个client
    pub fn new() -> Result<BcosSDK, KissError> {
        let configfile = "conf/config.toml";
        BcosSDK::new_from_config(configfile)
    }

    ///优雅退出
    pub fn finish(&mut self) {
        self.netclient.finish();
    }

    pub fn deploy_hexcode(&mut self, hexcode: &str) -> Result<JsonValue, KissError> {
        let block_limit = self.getBlockLimit()?;
        let to_address = "".to_string();
        let tx = self.make_transaction(&to_address, &hexcode, block_limit);
        let groupid = self.config.chain.groupid;
        let cmd = "sendRawTransaction";
        let rawdata = self.encode_sign_raw_tx(&tx.unwrap())?;
        let hexdata = hex::encode(rawdata);
        let paramobj = json!([groupid, hexdata]);
        let value = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(value)
    }
    //-----------------------------------------------------------------------------------
    ///部署合约，输入合约的bin文件，以及构造函数所需的参数，将构造函数参数后附在最后。部署完成后返回Json或错误信息
    ///参数用contractABI的构造函数encode_constructor_input构建
    pub fn deploy_file(&mut self, binfile: &str, params: &str) -> Result<JsonValue, KissError> {
        let hexcode = fileutils::readstring(binfile)?;
        let codewithparam = format!("{}{}", hexcode, params); //追加参数
        self.deploy_hexcode(codewithparam.as_str())
    }
    //传入已经加载的二进制合约代码，合约名，字符串数组类型的参数，部署合约
    pub fn deploy_code_withparam(
        &mut self,
        hexcode: &str,
        contractname: &str,
        params_array: &[String],
    ) -> Result<JsonValue, KissError> {
        //let binfile = format!("{}/{}.bin",self.config.contract.contractpath,contract_name.to_string())?;
        let contract = ContractABI::new_by_name(
            contractname,
            self.config.contract.contractpath.as_str(),
            &CommonHash::crypto_to_hashtype(&self.config.chain.crypto),
        )?;
        let paramcode = contract
            .encode_construtor_input("".as_bytes().to_vec(), &params_array, true)
            .unwrap();
        let codewithparam = format!("{}{}", hexcode, paramcode); //追加参数
        self.deploy_hexcode(codewithparam.as_str())
    }

    //传入合约名，从bin文件加载合约代码，拼装字符串数组类型的参数，部署合约
    pub fn deploy_withparam(
        &mut self,
        contractname: &str,
        params_array: &[String],
    ) -> Result<JsonValue, KissError> {
        let contract = ContractABI::new_by_name(
            contractname,
            self.config.contract.contractpath.as_str(),
            &CommonHash::crypto_to_hashtype(&self.config.chain.crypto),
        )?;
        let paramcode = contract
            .encode_construtor_input("".as_bytes().to_vec(), &params_array, true)
            .unwrap();
        let binfile = format!(
            "{}/{}.bin",
            self.config.contract.contractpath,
            contractname.to_string()
        );
        self.deploy_file(binfile.as_str(), paramcode.as_str())
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#call
    ///  Request
    /// curl -X POST --data '{"jsonrpc":"2.0","method":"call","params":[1,{"from":"0x6bc952a2e4db9c0c86a368d83e9df0c6ab481102","to":"0xd6f1a71052366dbae2f7ab2d5d5845e77965cf0d","value":"0x1","data":"0x3"}],"id":1}' http://127.0.0.1:8545 |jq
    ///
    /// // Result
    /// {
    ///     "id": 1,
    ///     "jsonrpc": "2.0",
    ///     "result": {
    ///         "currentBlockNumber": "0xb",
    ///         "output": "0x",
    ///         "status": "0x0"
    ///     }
    /// }
    pub fn call(
        &mut self,
        contract: &ContractABI,
        address: &str,
        method: &str,
        params: &[String],
    ) -> Result<JsonValue, KissError> {
        let groupid = self.config.chain.groupid;
        let from = hex::encode(&self.account.address);
        let to = address;
        let res = contract.encode_function_input_to_abi(method, params, true);
        let rawdata = match res {
            Ok(data) => data,
            Err(e) => {
                return kisserr!(
                    KissErrKind::EFormat,
                    "encode call {:?} error {:?}",
                    method,
                    e
                )
            }
        };
        let paramobj = json!([groupid,
        {"from":from,
        "to":to,
        "data":rawdata,
        "value":0
        }]);
        let value = self.netclient.rpc_request_sync("call", &paramobj)?;
        Ok(value)
    }


    ///引用客户端配置，构建一个未签名的交易
    pub fn make_transaction(
        &self,
        to_address: &str,
        txinput: &str,
        block_limit_i32: u32,
    ) -> Option<BcosTransaction> {
        let randid: u64 = rand::random();
        let chainid = self.config.chain.chainid;
        let groupid = self.config.chain.groupid;
        Option::from(BcosTransaction {
            to_address: crate::bcossdk::bcostransaction::encode_address(to_address),
            random_id: U256::from(randid),
            gas_price: U256::from(30000000),
            gas_limit: U256::from(30000000),
            block_limit: U256::from(block_limit_i32),
            value: U256::from(0),
            data: hex::decode(txinput).unwrap(),
            fisco_chain_id: U256::from(chainid),
            group_id: U256::from(groupid),
            extra_data: b"".to_vec(),
            hashtype: self.hashtype.clone(), //sdk在这里把hash算法配置传给了transaction
        })
    }

    ///根据配置选择签名算法实现
    pub fn pick_signer(&self) -> &dyn ICommonSigner {
        match self.config.chain.crypto {
            BcosCryptoKind::ECDSA => {
                let signer = self.ecdsasigner.as_ref().unwrap();
                printlnex!("pick signer {:?}", signer.account.to_hexdetail());
                signer
            }
            BcosCryptoKind::GM => {
                let signer = self.gmsigner.as_ref().unwrap();
                printlnex!("pick signer {:?}", signer.account.to_hexdetail());
                signer
            }
        }
    }

    ///引用客户端已经配置好的account，对交易进行签名
    pub fn encode_sign_raw_tx(&self, tx: &BcosTransaction) -> Result<Vec<u8>, KissError> {
        let signer = self.pick_signer();

        let txsig = BcosTransactionWithSig::sign(signer, tx)?;
        let rawdata = txsig.encode();
        Ok(rawdata)
    }
    ///输入已经解析好的param，直接根据组包，调用合约
    pub fn send_raw_transaction_withtokenparam(
                &mut self,
        contract: &ContractABI,
        to_address: &str,
        methodname: &str,
        params: &[Token],
    )-> Result<JsonValue, KissError>
    {
        let block_limit = self.getBlockLimit()?;
        let function = contract.find_function_unwrap(methodname) ?;
        //println!("function : {:?}",function);
        let txinput = ContractABI::encode_function_input_to_abi_by_tokens(&function, params, &self.hashtype)?;
        let tx = self.make_transaction(to_address, &hex::encode(txinput), block_limit);
        let groupid = self.config.chain.groupid;
        let cmd = "sendRawTransaction";
        let rawdata = self.encode_sign_raw_tx(&tx.unwrap())?;
        let hexdata = hex::encode(rawdata);
        let paramobj = json!([groupid, hexdata]);
        let value = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(value)
    }
    ///输入字符串数组类型的param,根据合约ABI解析并组包，调用合约
    pub fn send_raw_transaction(
        &mut self,
        contract: &ContractABI,
        to_address: &str,
        methodname: &str,
        params: &[String],
    ) -> Result<JsonValue, KissError> {
        let txinput = contract.convert_function_input_str_to_token(methodname, params, true)?;
        //println!("txinput tokens len {}, {:?},",txinput.len(),txinput);
        let value = self.send_raw_transaction_withtokenparam(contract, to_address, methodname, &txinput)?;
        Ok(value)
    }

    ///传入的是类型已经按ABI好的token
    pub fn sendRawTransactionGetReceiptWithTokenParam(
        &mut self,
        contract: &ContractABI,
        to_address: &str,
        methodname: &str,
        params: &[Token],
    ) -> Result<JsonValue, KissError> {
        let response = self.send_raw_transaction_withtokenparam(&contract, &to_address, methodname, params)?;
        let txhash = response["result"].as_str().unwrap();
        self.try_getTransactionReceipt(txhash, 3, false)
    }

    ///简单封装下同步的发送交易且获得回执的方法。默认等待1s,这是个非常常用的方法，尤其是用于demo时
    pub fn sendRawTransactionGetReceipt(
        &mut self,
        contract: &ContractABI,
        to_address: &str,
        methodname: &str,
        params: &[String],
    ) -> Result<JsonValue, KissError> {
        let response = self.send_raw_transaction(&contract, &to_address, methodname, &params)?;
        println!("response {:?}",response);
        let txhash = response["result"].as_str().unwrap();
        self.try_getTransactionReceipt(txhash, 3, false)
    }
    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#sendrawtransactionandgetproof
    pub fn sendRawTransactionAndGetProof(
        &mut self,
        contract: &ContractABI,
        to_address: &str,
        methodname: &str,
        params: &[String],
    ) -> Result<JsonValue, KissError> {
        let block_limit = self.getBlockLimit()?;
        let txinput = contract.encode_function_input_to_abi(methodname, params, true)?;
        let tx = self.make_transaction(to_address, &txinput.as_str(), block_limit);
        let groupid = self.config.chain.groupid;
        let cmd = "sendRawTransactionAndGetProof";
        let rawdata = self.encode_sign_raw_tx(&tx.unwrap())?;
        let hexdata = hex::encode(&rawdata);
        let paramobj = json!([groupid, hexdata]);
        let value = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(value)
    }

    ///编译合约。传入合约名字和配置文件路径
    ///因为不需要连接节点，纯本地运行，采用静态方法实现，避免加载各种库，也无需连接网络
    pub fn compile(contract_name:&str,configfile:&str)->Result<Output,KissError> {

        let config = ClientConfig::load(configfile)?;
        let mut solc_path = match config.chain.crypto {
            BcosCryptoKind::ECDSA => {
                config.contract.solc
            },
            BcosCryptoKind::GM => {
                config.contract.solcgm
            }
        };
        if cfg!(target_os = "windows")
        {
            solc_path = format!("{}.exe",solc_path);
        }

         if ! Path::new(solc_path.as_str()).exists()   {
            return kisserr!(KissErrKind::EFileMiss,"solc [{}] is not exists,check the solc setting in config file [{}]",solc_path,configfile);
         }

        let mut solfullpath = PathBuf::from(&config.contract.contractpath);
        let options = ["--abi", "--bin", "--bin-runtime", "--overwrite","--hashes"];
        solfullpath = solfullpath.join(format!("{}.sol", contract_name));
         if !  solfullpath.exists()  {
            return kisserr!(KissErrKind::EFileMiss,"contract solfile [{}] is not exists,check the config setting in  [{}]->contractpath[{}]",
                solfullpath.to_str().unwrap(),configfile,config.contract.contractpath);
         }
        info!("compile sol  {} ,use solc {},outputdir:{} options: {:?} ",
                 solfullpath.to_str().unwrap(), solc_path, config.contract.contractpath.as_str(), options);
        let outputres = Command::new(solc_path).
            args(&options)
            .arg("-o").arg(config.contract.contractpath.as_str())
            .arg(solfullpath.to_str().unwrap())
            .output();
        info!("compile result : {:?}", &outputres);
        match outputres {
            Ok(out)=>{return Ok(out)}
            Err(e)=>{return kisserr!(KissErrKind::Error,"compile [{}] error :{:?}",contract_name,e)}
        }

    }

}
