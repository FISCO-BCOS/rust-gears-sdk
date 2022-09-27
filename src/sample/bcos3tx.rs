use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_long;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use libc::{c_char, c_ulong, wchar_t};
use libc::c_int;
use libc::c_void;
use libloading::{Library, Symbol};
use serde_json::{json, Value as JsonValue};
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkfuture::Bcos3SDKFuture;

use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkwrapper::{*};
use fisco_bcos_rust_gears_sdk::str2p;

use crate::bcossdk::abi_parser::ABIParser;
use crate::bcossdk::commonhash::HashType;
use crate::bcossdk::contractabi::ContractABI;
//Result as JsonResult
use crate::bcossdk::kisserror::KissError;
use crate::bcossdk::liteutils::datetime_str;
use crate::ClientConfig;

fn call(p_sdk: *const c_void, group: &str, seq: u64, to: &str, funcname: &str, paramsvec: &Vec<String>, abi: &ContractABI) {
    unsafe {
        let functiondata = abi.encode_function_input_to_abi("get", &paramsvec, true).unwrap();
        let future_context = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(), "call", "do call get");
        bcos_rpc_call(p_sdk, str2p!(group), 0 as *const c_char, str2p!(to),
                      str2p!(functiondata),
                      Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                      Bcos3SDKFuture::to_c_ptr(&future_context),
        );

        let response = future_context.wait().unwrap();
        let result = response.get_result().unwrap();
        let resultdata = result["output"].as_str().unwrap();
        let outputres = abi.decode_output_byname("get", resultdata);
        println!("call output {:?}", outputres);
    }
}

#[cfg(feature = "bcos3sdk_ffi")]
fn test_bcos3sdk_ffi_tx() {
    unsafe {
        let group = "group0";
        let p_sdk = init_bcos3sdk_lib("bcos3sdklib/bcos3_sdk_config.ini");
        let chainid = bcos_sdk_get_group_chain_id(p_sdk, str2p!(group));
        let mut wasm: c_int = 0;
        let mut sm_cryto: c_int = 0;
        bcos_sdk_get_group_wasm_and_crypto(p_sdk, str2p!(group),
                                           &mut wasm as *mut c_int, &mut sm_cryto as *mut c_int);

        let blocklimit = bcos_rpc_get_block_limit(p_sdk, str2p!(group));
        println!(">>> blocklimit {}", blocklimit);
        let chainid_str = CStr::from_ptr(chainid);
        println!("ChainID : {:?}", chainid_str);
        //  0:ecdsa  1:sm
        //let key_pair = bcos_sdk_create_keypair(0);
        let key = "7a94d9793bcc38f533c6e15d8ef9c557e8ead2d3f86e9ac1178ce56b2815f86b";
        let key_pair = bcos_sdk_create_keypair_by_hex_private_key(0, str2p!(key));
        let contract_address = "2237d46dada4c0306699555fc0bc6a31da29e4b4";

        let abi = ContractABI::new_by_name("HelloWorld",
                                           "./contracts", &HashType::WEDPR_KECCAK).unwrap();
        let paramsvec = vec!();
        call(p_sdk, group, Bcos3SDKFuture::next_seq(), &contract_address, "get", &paramsvec, &abi);

        let future_context = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(), "number", "");
        bcos_rpc_get_block_number(p_sdk, str2p!(group), 0 as *const c_char,
                                  Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                  Bcos3SDKFuture::to_c_ptr(&future_context),
        );
        let response = future_context.wait();
        println!("get block number {:?}", response);


        //let paramsvec = vec!(format!("Test string for helloworld: {}", datetime_str()));
        let paramsvec = vec!(format!("abcdefg"));
        let functiondata = abi.encode_function_input_to_abi("set", &paramsvec, true).unwrap();
        println!("function data len {}, {}", functiondata.len(), functiondata);
        let future_context = Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(), "set", "set data");
        let p_txhash = Box::into_raw(Box::new(0 as *mut c_char));
        let p_signed_tx = Box::into_raw(Box::new(0 as *mut c_char));
        println!("p_txhash {:?}", p_txhash);
        println!("blocklimit {}", blocklimit);
        println!("functiondata :{}", functiondata);
        //blocklimit = 10000000000000000;
        bcos_sdk_create_signed_transaction(key_pair, str2p!(group), chainid as *const c_char,
                                           str2p!(contract_address), str2p!(functiondata.as_str()), str2p!(""),
                                            blocklimit, 0,
                                           p_txhash,
                                           p_signed_tx,
        );
        println!("last error {:?}", CStr::from_ptr(bcos_sdk_get_last_error_msg()));

        let txhash_str = CStr::from_ptr(*p_txhash);
        let signed_tx_str = CStr::from_ptr(*p_signed_tx);


        println!("txhash {:?}", txhash_str);
        println!("signed_tx {:?}", signed_tx_str);
        bcos_rpc_send_transaction(p_sdk, str2p!(group), 0 as *const c_char, *p_signed_tx, 0,
                                  Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                                  Bcos3SDKFuture::to_c_ptr(&future_context),
        );
        bcos_sdk_c_free(*p_txhash as *const c_void);
        bcos_sdk_c_free(*p_signed_tx as *const c_void);


        let response = future_context.wait().unwrap();
        response.display();
        let result = response.get_result().unwrap();
        println!("result is {:?}", result);

        let input = abi.decode_input_for_tx(result["input"].as_str().unwrap());
        let output = abi.decode_output_byname("set", result["output"].as_str().unwrap());
        let logs = abi.parse_receipt_logs(&result["logEntries"]);
        println!("input {:?}", input);
        println!("output {:?}", output);
        println!("logs {:?}", logs);


        println!("ready to quit");
        thread::sleep(Duration::from_secs(1));
        bcos_sdk_stop(p_sdk);
        bcos_sdk_destroy(p_sdk);
        println!("Ok to quit");
    }
}

#[cfg(feature = "bcos3sdk_ffi")]
pub fn test_bcos3tx() {
    test_bcos3sdk_ffi_tx();

    let configfile = "conf/config.toml";
    let config = ClientConfig::load(configfile);
    println!("{:?}",config);
}