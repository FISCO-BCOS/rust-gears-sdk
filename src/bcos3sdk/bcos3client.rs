/*包装一次bcos3sdkwrapper里的ffi方法，映射成rust的典型写法，并将各方法采用同步方式实现
  异步方式有待todo，需要异步方式的，可以参考回调函数定义来实现
*/

use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicU64, Ordering};

use encoding::{DecoderTrap, Encoding};
use encoding::all::GBK;
use libc::{c_char, c_int, c_longlong, c_void};
use serde_json::Value as JsonValue;

use crate::{kisserr, kisserrcode, str2p};
use crate::bcos3sdk::bcos3sdk_ini::Bcos3sdkIni;
use crate::bcos3sdk::bcos3sdkfuture::Bcos3SDKFuture;
use crate::bcos3sdk::bcos3sdkwrapper::*;
use crate::bcos3sdk::bcos3sdkwrapper::bcos3sdk_def::*;
use crate::bcossdkutil::accountutil::{account_from_pem, BcosAccount};
use crate::bcossdkutil::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use crate::bcossdkutil::commonhash::{CommonHash, HashType};
use crate::bcossdkutil::contractabi::ContractABI;
use crate::bcossdkutil::fileutils;
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};

//定义一个结构体，简单包装sdk指针，有待扩展
pub struct Bcos3Client {
    pub crytotype: i32,
    pub hashtype: HashType,
    pub keypair: *const c_void,
    pub account: BcosAccount,
    pub config: ClientConfig,
    pub bcos3sdkini: Bcos3sdkIni,
    pub sdk: *const c_void,
    pub clientname: String,
    pub group: String,
    pub chainid: String,
    pub node: String,
    pub reqcounter: AtomicU64,
}

impl Bcos3Client {
    pub fn get_full_name(&self) -> String {
        format!("{}-{}-{}", self.clientname, self.chainid, self.group)
    }
    pub fn getLastError() -> i32 {
        unsafe {
            let errcode = bcos_sdk_get_last_error();
            return errcode as i32;
        }
    }
    pub fn get_info(&self) -> String {
        let info = format!("chain:[{}],group:[{}],crypto:[{}],account:[0x{}],peers:[{:?}]\n{}",
                           self.chainid, self.group, self.crytotype,
                           hex::encode(&self.account.address),
                           self.bcos3sdkini.peers,
                           self.getVersion()
        );
        return info;
    }

    //驼峰式命名用于包装native c sdk 接口的方法，没啥原因，就是做个区分
    pub fn getLastErrMessage() -> String {
        unsafe {
            let mut msgstr: String = "".to_string();
            let last_err_msg = bcos_sdk_get_last_error_msg();
            if last_err_msg != (0 as *mut c_char) {
                let errcstr = CStr::from_ptr(last_err_msg);
                let errcstr_tostr = errcstr.to_str();
                //这里要处理下编码，rust默认是UTF-8,如果不ok，那就是其他字符集
                if errcstr_tostr.is_ok() {
                    msgstr = errcstr_tostr.unwrap().to_string();
                } else {
                    //强行尝试对CStr对象进行GBK解码,采用replace策略
                    //todo: 如果在使用其他编码的平台上依旧有可能失败，得到空消息，但不会抛异常了
                    let alter_msg = GBK.decode(errcstr.to_bytes(), DecoderTrap::Replace);
                    // let alter_msg = encoding::all::UTF_8.decode(errcstr.to_bytes(),DecoderTrap::Replace);
                    if alter_msg.is_ok() {
                        msgstr = alter_msg.unwrap();
                    }
                }
            }
            return msgstr;
        }
    }
    pub fn new(configfile: &str) -> Result<Self, KissError> {
        unsafe {
            let config = ClientConfig::load(configfile).unwrap();
            let sdk = init_bcos3sdk_lib(config.bcos3.sdk_config_file.as_str());
            if sdk == 0 as *const c_void {
                return kisserr!(KissErrKind::Error,"BCOS3 C LIB is NOT init;ERROR:{}:{}",bcos_sdk_get_last_error(),Bcos3Client::getLastErrMessage());
            }
            if bcos_sdk_get_last_error() != 0 {
                return kisserr!(KissErrKind::Error,"BCOS3 C LIB init/start error {}",Bcos3Client::getLastErrMessage());
            }
            let bcos3sdkini = Bcos3sdkIni::load(config.bcos3.sdk_config_file.as_str())?;
            let mut cryptotype = 0;
            let account =
                account_from_pem(config.common.accountpem.as_str(), &config.common.crypto)?;
            let privkey = hex::encode(&account.privkey);
            let hashtype = CommonHash::crypto_to_hashtype(&config.common.crypto);

            match &config.common.crypto {
                BcosCryptoKind::ECDSA => {
                    cryptotype = 0;
                }
                BcosCryptoKind::GM => {
                    cryptotype = 1;
                }
            }
            let keypair =
                bcos_sdk_create_keypair_by_hex_private_key(cryptotype, str2p!(privkey.as_str()));

            let client = Bcos3Client {
                clientname: "BCOS3".to_string(),
                crytotype: cryptotype,
                hashtype: hashtype,
                sdk: sdk,
                group: config.bcos3.group.clone(),
                chainid: "chain0".to_string(),
                config: config,
                bcos3sdkini: bcos3sdkini,
                keypair: keypair,
                account: account,
                node: "".to_string(),
                reqcounter: AtomicU64::new(0),
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
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getBlockNumber",
                format!("").as_str(),
            );
            bcos_rpc_get_block_number(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );

            let result = cbfuture.wait_result()?;
            let num_option = result.as_u64();

            match num_option {
                Some(num) => {
                    return Ok(num);
                }
                None => {
                    return kisserr!(KissErrKind::Error, "getBlockNumber from result error");
                }
            }
        }
    }

    pub fn getVersion(&self) -> String {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let version = bcos_sdk_version();
            let s_v = CStr::from_ptr(version).to_str();
            if s_v.is_ok() {
                return s_v.unwrap().to_string();
            }
            return "[UNKNOW VERSION]".to_string();
        }
    }
    pub fn getBlocklimit(&self) -> Result<u64, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let new_blockLimit = bcos_rpc_get_block_limit(self.sdk, str2p!(self.group.as_str()));
            Ok(new_blockLimit as u64)
        }
    }

    pub fn getPbftView(&self) -> Result<u64, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getPbftView",
                format!("").as_str(),
            );
            bcos_rpc_get_pbft_view(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            let result = cbfuture.wait_result()?;
            let num_option = result.as_u64();
            match num_option {
                Some(num) => {
                    return Ok(num);
                }
                None => {
                    return kisserr!(KissErrKind::Error, "getPbftView from result error");
                }
            }
        }
    }

    pub fn getSealerList(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getPbftView",
                format!("").as_str(),
            );
            bcos_rpc_get_sealer_list(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getObserverList(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getObserverList",
                format!("").as_str(),
            );
            bcos_rpc_get_observer_list(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getConsensusStatus(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getConsensusStatus",
                format!("").as_str(),
            );
            bcos_rpc_get_consensus_status(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getSyncStatus(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getSyncStatus",
                format!("").as_str(),
            );
            bcos_rpc_get_sync_status(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getPeers(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getPeers",
                format!("").as_str(),
            );
            bcos_rpc_get_peers(
                self.sdk,
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getGroupPeers(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getGroupPeers",
                format!("").as_str(),
            );
            bcos_rpc_get_group_peers(
                self.sdk,
                str2p!(self.group.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getGroupList(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getGroupList",
                format!("").as_str(),
            );
            bcos_rpc_get_group_list(
                self.sdk,
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getBlockByHash(
        &self,
        block_hash: &str,
        only_header: u32,
        only_tx_hash: u32,
    ) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getBlockByHash",
                format!("").as_str(),
            );
            bcos_rpc_get_block_by_hash(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                str2p!(block_hash),
                only_header as c_int,
                only_tx_hash as c_int,
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getBlockByNumber(
        &self,
        num: u64,
        only_header: u32,
        only_tx_hash: u32,
    ) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getBlockByNumber",
                format!("").as_str(),
            );
            bcos_rpc_get_block_by_number(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                num as c_longlong,
                only_header as c_int,
                only_tx_hash as c_int,
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getBlockHashByNumber(&self, num: u64) -> Result<String, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getBlockHashByNumber",
                format!("").as_str(),
            );
            bcos_rpc_get_block_hash_by_number(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                num as c_longlong,
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );

            let v = cbfuture.wait_result()?;
            //println!("block v {:?}",v);
            let hash = v.as_str().unwrap();
            return Ok(hash.to_string());
        }
    }
    pub fn getTotalTransactionCount(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getTotalTransactionCount",
                format!("").as_str(),
            );
            bcos_rpc_get_total_transaction_count(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getTransactionByHash(&self, hash: &str, proof: i32) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getTransactionByHash",
                format!("").as_str(),
            );
            bcos_rpc_get_transaction(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                str2p!(hash),
                proof as c_int,
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getTransactionReceipt(&self, hash: &str, proof: i32) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getTransactionReceipt",
                format!("").as_str(),
            );
            bcos_rpc_get_transaction_receipt(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                str2p!(hash),
                proof,
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getPendingTxSize(&self) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getPendingTxSize",
                format!("").as_str(),
            );
            bcos_rpc_get_pending_tx_size(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getCode(&self, address: &str) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture =
                Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(), "getCode", format!("").as_str());
            bcos_rpc_get_code(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                str2p!(address),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn getSystemConfigByKey(&self, key: &str) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "getSystemConfigByKey",
                format!("").as_str(),
            );
            bcos_rpc_get_system_config_by_key(
                self.sdk,
                str2p!(self.group.as_str()),
                str2p!(self.node.as_str()),
                str2p!(key),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            return cbfuture.wait_result();
        }
    }

    pub fn call(
        &self,
        to: &str,
        funcname: &str,
        paramsvec: &Vec<String>,
        abi: &ContractABI,
    ) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let functiondata = abi
                .encode_function_input_to_abi(funcname, &paramsvec, true)
                .unwrap();
            let seq = 0;
            let cbfuture = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(), funcname, "do call");
            bcos_rpc_call(
                self.sdk,
                str2p!(self.group.as_str()),
                0 as *const c_char,
                str2p!(to),
                str2p!(functiondata),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );

            let result = cbfuture.wait_result()?;
            Ok(result)
        }
    }

    pub fn sendRawTransaction(
        &self,
        to_address: &str,
        methodname: &str,
        functiondata: &str,
    ) -> Result<JsonValue, KissError> {
        self.reqcounter.fetch_add(1, Ordering::Relaxed);
        unsafe {
            let cbfuture = Bcos3SDKFuture::create(
                Bcos3SDKFuture::next_seq(),
                "sendTransction",
                format!("{}", methodname).as_str(),
            );

            //println!("function data len {}, {}", functiondata.len(), functiondata);
            let p_txhash = Box::into_raw(Box::new(0 as *mut c_char));
            let p_signed_tx = Box::into_raw(Box::new(0 as *mut c_char));
            let blocklimit = bcos_rpc_get_block_limit(self.sdk, str2p!(self.group.as_str()));

            bcos_sdk_create_signed_transaction(
                self.keypair,
                str2p!(self.group.as_str()),
                str2p!(self.chainid.as_str()),
                str2p!(to_address),
                str2p!(functiondata),
                str2p!(""),
                blocklimit,
                0,
                p_txhash,
                p_signed_tx,
            );
            let lasterr = Bcos3Client::getLastError();
            if lasterr != 0 {
                let last_err_msg = Bcos3Client::getLastErrMessage();
                return kisserrcode!(KissErrKind::Error, lasterr as i64, "{}", last_err_msg);
            }
            let txhash_str = CStr::from_ptr(*p_txhash);
            let signed_tx_str = CStr::from_ptr(*p_signed_tx);

            //println!("txhash {:?}", txhash_str);
            //println!("signed_tx {:?}", signed_tx_str);
            bcos_rpc_send_transaction(
                self.sdk,
                str2p!(self.group.as_str()),
                0 as *const c_char,
                *p_signed_tx,
                0,
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            bcos_sdk_c_free(*p_txhash as *const c_void);
            bcos_sdk_c_free(*p_signed_tx as *const c_void);

            let result = cbfuture.wait_result()?;

            Ok(result)
        }
    }

    pub fn sendTransaction(
        &self,
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
