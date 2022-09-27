/*包装一次bcos3sdkwrapper里的ffi方法，映射成rust的典型写法，并将各方法采用同步方式实现
  异步方式有待todo，需要异步方式的，可以参考回调函数定义来实现
*/

use std::ffi::{CStr, CString};

use libc::{c_char, c_int, c_longlong, c_void};
use serde_json::Value as JsonValue;

use crate::{kisserr, kisserrcode, str2p};
use crate::bcos3sdk::bcos3sdkfuture::Bcos3SDKFuture;
#[cfg(feature = "bcos3sdk_ffi")]
use crate::bcos3sdk::bcos3sdkwrapper::{*};
use crate::bcossdk::{fileutils};
use crate::bcossdk::accountutil::account_from_pem;
use crate::bcossdk::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use crate::bcossdk::commonhash::{CommonHash, HashType};
use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::kisserror::{KissErrKind, KissError};

//定义一个结构体，简单包装sdk指针，有待扩展
pub struct Bcos3Client {
    pub crytotype: i32,
    pub hashtype: HashType,
    pub keypair: *const c_void,
    pub config: ClientConfig,
    pub sdk: *const c_void,
    pub clientname: String,
    pub group: String,
    pub chainid: String,
    pub node: String,
}

impl Bcos3Client {
    pub fn get_full_name(&self) -> String {
        format!("{}-{}-{}", self.clientname, self.chainid, self.group)
    }

    pub fn new(configfile: &str) -> Result<Self, KissError> {
        unsafe {
            let config = ClientConfig::load(configfile).unwrap();
            let sdk = init_bcos3sdk_lib(config.bcos3.sdk_config_file.as_str());
            let mut cryptotype = 0;
            let account = account_from_pem(config.common.accountpem.as_str(), &config.common.crypto)?;
            let privkey = hex::encode(account.privkey);
            let hashtype = CommonHash::crypto_to_hashtype(&config.common.crypto);
            match &config.common.crypto {
                BcosCryptoKind::ECDSA => {
                    cryptotype = 0;
                }
                BcosCryptoKind::GM => {
                    cryptotype = 1;
                }
            }
            let keypair = bcos_sdk_create_keypair_by_hex_private_key
                (cryptotype, str2p!(privkey.as_str()));
            let client = Bcos3Client {
                clientname: "BCOS3".to_string(),
                crytotype: cryptotype,
                hashtype: hashtype,
                sdk: sdk,
                group: config.bcos3.group.clone(),
                chainid: "chain0".to_string(),
                config: config,
                keypair: keypair,
                node: "".to_string(),
            };
            Ok(client)
        }
    }

    pub fn finish(&mut self) {
        unsafe {
            if self.sdk == 0 as *const c_void {
                return;
            }
            bcos_sdk_stop(self.sdk);
            bcos_sdk_destroy(self.sdk);
            self.sdk = 0 as *const c_void;
        }
    }

    pub fn getBlockNumber(&self) -> Result<u64, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getBlockNumber", format!("").as_str());
            bcos_rpc_get_block_number(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                      Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                      Bcos3SDKFuture::to_c_ptr(&cbfuture));

            let result = cbfuture.wait_result()?;
            let num_option = result.as_u64();

            match num_option {
                Some(num) => { return Ok(num); }
                None => { return kisserr!(KissErrKind::Error,"getBlockNumber from result error"); }
            }
        }
    }

    pub fn getVersion(&self) -> Result<String, KissError> {
        unsafe {
            let version = bcos_sdk_version();
            let s_v = CStr::from_ptr(version).to_str().unwrap().to_string();
            Ok(s_v)
        }
    }
    pub fn getBlocklimit(&self) -> Result<u64, KissError> {
        unsafe {
            let new_blockLimit = bcos_rpc_get_block_limit(self.sdk, str2p!(self.group.as_str()));
            Ok(new_blockLimit as u64)
        }
    }

    pub fn getPbftView(&self) -> Result<u64, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getPbftView", format!("").as_str());
            bcos_rpc_get_pbft_view(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                   Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                   Bcos3SDKFuture::to_c_ptr(&cbfuture));
            let result = cbfuture.wait_result()?;
            let num_option = result.as_u64();
            match num_option {
                Some(num) => { return Ok(num); }
                None => { return kisserr!(KissErrKind::Error,"getPbftView from result error"); }
            }
        }
    }


    pub fn getSealerList(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getPbftView", format!("").as_str());
            bcos_rpc_get_sealer_list(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()), Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                     Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getObserverList(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getObserverList", format!("").as_str());
            bcos_rpc_get_observer_list(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()), Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                       Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getConsensusStatus(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getConsensusStatus", format!("").as_str());
            bcos_rpc_get_consensus_status(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()), Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                          Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getSyncStatus(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getSyncStatus", format!("").as_str());
            bcos_rpc_get_sync_status(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()), Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                     Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getPeers(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getPeers", format!("").as_str());
            bcos_rpc_get_peers(self.sdk, Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getGroupPeers(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getGroupPeers", format!("").as_str());
            bcos_rpc_get_group_peers(self.sdk, str2p!(self.group.as_str()), Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                     Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getGroupList(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getGroupList", format!("").as_str());
            bcos_rpc_get_group_list(self.sdk, Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getBlockByHash(&self, block_hash: &str, only_header: u32, only_tx_hash: u32) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getBlockByHash", format!("").as_str());
            bcos_rpc_get_block_by_hash(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                       str2p!(block_hash),
                                       only_header as c_int,
                                       only_tx_hash as c_int,
                                       Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getBlockByNumber(&self, num: u64, only_header: u32, only_tx_hash: u32) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getBlockByNumber", format!("").as_str());
            bcos_rpc_get_block_by_number(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                         num as c_longlong,
                                         only_header as c_int,
                                         only_tx_hash as c_int,
                                         Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                         Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getBlockHashByNumber(&self, num: u64) -> Result<String, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getBlockHashByNumber", format!("").as_str());
            bcos_rpc_get_block_hash_by_number(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                              num as c_longlong,
                                              Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));

            let v = cbfuture.wait_result()?;
            //println!("block v {:?}",v);
            let hash = v.as_str().unwrap();
            return Ok(hash.to_string());
        }
    }
    pub fn getTotalTransactionCount(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getTotalTransactionCount", format!("").as_str());
            bcos_rpc_get_total_transaction_count(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                                 Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }


    pub fn getTransactionByHash(&self, hash: &str, proof: i32) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getTransactionByHash", format!("").as_str());
            bcos_rpc_get_transaction(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                     str2p!(hash),
                                     proof as c_int,
                                     Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                     Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getTransactionReceipt(&self, hash: &str, proof: i32) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getTransactionReceipt", format!("").as_str());
            bcos_rpc_get_transaction_receipt(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                             str2p!(hash),
                                             proof,
                                             Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getPendingTxSize(&self) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getPendingTxSize", format!("").as_str());
            bcos_rpc_get_pending_tx_size(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                         Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }

    pub fn getCode(&self, address: &str) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getCode", format!("").as_str());
            bcos_rpc_get_code(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                              str2p!(address),
                              Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }


    pub fn getSystemConfigByKey(&self, key: &str) -> Result<JsonValue, KissError> {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "getSystemConfigByKey", format!("").as_str());
            bcos_rpc_get_system_config_by_key(self.sdk, str2p!(self.group.as_str()), str2p!(self.node.as_str()),
                                              str2p!(key),
                                              Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC, Bcos3SDKFuture::to_c_ptr(&cbfuture));
            return cbfuture.wait_result();
        }
    }


    pub fn call(&self, to: &str, funcname: &str, paramsvec: &Vec<String>, abi: &ContractABI) -> Result<JsonValue, KissError> {
        unsafe {
            let functiondata = abi.encode_function_input_to_abi(funcname, &paramsvec, true).unwrap();
            let seq = 0;
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  funcname, "do call");
            bcos_rpc_call(self.sdk, str2p!(self.group.as_str()), 0 as *const c_char, str2p!(to),
                          str2p!(functiondata),
                          Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                          Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );

            let response = cbfuture.wait().unwrap();
            let result = response.get_result()?;
            Ok(result)
        }
    }

    pub fn sendRawTransaction(&self, to_address: &str, methodname: &str, functiondata: &str) -> Result<JsonValue, KissError>
    {
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(),
                                                  "sendTransction", format!("{}", methodname).as_str());

            //println!("function data len {}, {}", functiondata.len(), functiondata);
            let p_txhash = Box::into_raw(Box::new(0 as *mut c_char));
            let p_signed_tx = Box::into_raw(Box::new(0 as *mut c_char));
            let blocklimit = bcos_rpc_get_block_limit(self.sdk, str2p!(self.group.as_str()));

            bcos_sdk_create_signed_transaction(self.keypair, str2p!(self.group.as_str()),
                                               str2p!(self.chainid.as_str()),
                                               str2p!(to_address),
                                               str2p!(functiondata),
                                               str2p!(""),
                                               blocklimit, 0,
                                               p_txhash,
                                               p_signed_tx,
            );
            let lasterr = bcos_sdk_get_last_error();
            if lasterr != 0 {
                let last_err_msg = bcos_sdk_get_last_error_msg();
                let msgstr = CStr::from_ptr(last_err_msg).to_str().unwrap();
                return kisserrcode!(KissErrKind::Error,lasterr as i64,"{}",msgstr);
            }

            let txhash_str = CStr::from_ptr(*p_txhash);
            let signed_tx_str = CStr::from_ptr(*p_signed_tx);


            //println!("txhash {:?}", txhash_str);
            //println!("signed_tx {:?}", signed_tx_str);
            bcos_rpc_send_transaction(self.sdk, str2p!(self.group.as_str()), 0 as *const c_char,
                                      *p_signed_tx,
                                      0,
                                      Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                      Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            bcos_sdk_c_free(*p_txhash as *const c_void);
            bcos_sdk_c_free(*p_signed_tx as *const c_void);


            let response = cbfuture.wait()?;
            let result = response.get_result().unwrap();

            Ok(result)
        }
    }

    pub fn sendTransaction(&self,
                           to_address: &str,
                           methodname: &str,
                           params: &[String],
                           contract: &ContractABI,
    ) -> Result<JsonValue, KissError> {
        let functiondata = contract.encode_function_input_to_abi(methodname, &params, true)?;
        let result = self.sendRawTransaction(to_address, methodname, functiondata.as_str());
        return result;
    }

    pub fn deploy_hexcode(&self, hexcode: &str) -> Result<JsonValue, KissError> {
        return self.sendRawTransaction("", "", hexcode);
    }
    //-----------------------------------------------------------------------------------
    ///部署合约，输入合约的bin文件，以及构造函数所需的参数，将构造函数参数后附在最后。部署完成后返回Json或错误信息
    ///参数用contractABI的构造函数encode_constructor_input构建
    pub fn deploy_file(&self, binfile: &str, params: &str) -> Result<JsonValue, KissError> {
        let hexcode = fileutils::readstring(binfile)?;
        let codewithparam = format!("{}{}", hexcode, params); //追加参数
        self.deploy_hexcode(codewithparam.as_str())
    }
    //传入已经加载的二进制合约代码，合约名，字符串数组类型的参数，部署合约
    pub fn deploy_code_withparam(
        &self,
        hexcode: &str,
        contractname: &str,
        params_array: &[String],
    ) -> Result<JsonValue, KissError> {
        //let binfile = format!("{}/{}.bin",self.config.common.contractpath,contract_name.to_string())?;
        let contract = ContractABI::new_by_name(
            contractname,
            self.config.common.contractpath.as_str(),
            &CommonHash::crypto_to_hashtype(&self.config.common.crypto),
        )?;
        let paramcode = contract
            .encode_construtor_input("".as_bytes().to_vec(), &params_array, true)
            .unwrap();
        let codewithparam = format!("{}{}", hexcode, paramcode); //追加参数
        self.deploy_hexcode(codewithparam.as_str())
    }

    //传入合约名，从bin文件加载合约代码，拼装字符串数组类型的参数，部署合约
    pub fn deploy_withparam(
        &self,
        contractname: &str,
        params_array: &[String],
    ) -> Result<JsonValue, KissError> {
        let contract = ContractABI::new_by_name(
            contractname,
            self.config.common.contractpath.as_str(),
            &CommonHash::crypto_to_hashtype(&self.config.common.crypto),
        )?;
        let paramcode = contract
            .encode_construtor_input("".as_bytes().to_vec(), &params_array, true)
            .unwrap();
        let binfile = format!(
            "{}/{}.bin",
            self.config.common.contractpath,
            contractname.to_string()
        );
        self.deploy_file(binfile.as_str(), paramcode.as_str())
    }
}