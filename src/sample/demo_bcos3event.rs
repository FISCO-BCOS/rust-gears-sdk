use std::collections::HashMap;
use std::ffi::CString;
use std::thread;
use std::time::Duration;

use libc::{c_char, c_void};
use tokio::time::Sleep;

use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3client::Bcos3Client;
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkfuture::Bcos3SDKFuture;
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkresponse::{
    bcos_sdk_c_struct_response, Bcos3SDKResponse,
};
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkwrapper::bcos3sdk_def::*;
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkwrapper::{EventSubParam, BCOS3SDK_CALLBACK_FUNC};
use fisco_bcos_rust_gears_sdk::bcossdkutil::commonhash::CommonHash;
use fisco_bcos_rust_gears_sdk::bcossdkutil::commonhash::HashType;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contractabi::ContractABI;
use fisco_bcos_rust_gears_sdk::bcossdkutil::contracthistory::ContractHistory;
use fisco_bcos_rust_gears_sdk::bcossdkutil::kisserror::{KissErrKind, KissError};
use fisco_bcos_rust_gears_sdk::bcossdkutil::liteutils::{datetime_str, json_str};
use fisco_bcos_rust_gears_sdk::str2p;

use crate::bcossdkutil::liteutils;
use crate::console::console_utils::display_receipt_logs;
use crate::Cli;

static mut g_counter: i32 = 0;

#[derive(Debug, Clone)]
struct EventCallbackContext {
    contractname: String,
    contractpath: String,
    hashtype: HashType,
}

pub extern "C" fn demo_bcos3_event_callback(resp: *const bcos_sdk_c_struct_response) {
    let response = Bcos3SDKResponse::from_callback(resp);
    unsafe {
        let pcontext = response.context_pointer as *const EventCallbackContext;
        println!("context is {:?}", (*pcontext));
        let contractname = (*pcontext).contractname.clone();
        let contextpath = (*pcontext).contractpath.clone();
        let contractabi = ContractABI::new_by_name(
            contractname.as_str(),
            contextpath.as_str(),
            &HashType::KECCAK,
        )
        .unwrap();

        let v = response.get_result().unwrap();

        g_counter = g_counter + 1;
        println!("{} >--- ASync callback ,v = {:?}", g_counter, v);

        if v.is_array() {
            println!("->display logs :");
            let logs = contractabi.parse_receipt_logs(&v).unwrap();
            display_receipt_logs(&logs);
        } else {
            println!("event sub got v : {:?}", v)
        }
    }
}

pub fn demo_event(cli: &Cli) -> Result<(), KissError> {
    unsafe {
        let mut mode: String = "sync".to_string(); //sync or async
        if cli.params.len() > 0 {
            mode = cli.params[0].clone();
        }
        let mut bcos3client = Bcos3Client::new(cli.default_configfile().as_str())?;
        let cbfuture =
            Bcos3SDKFuture::create(Bcos3SDKFuture::next_seq(), "eventsub", format!("").as_str());
        let contractname = "HelloWorld";
        let methodname = "onset";
        let contractabi = ContractABI::new_by_name(
            contractname,
            bcos3client.config.common.contractpath.as_str(),
            &bcos3client.hashtype,
        )?;

        let mut event_sub_param = EventSubParam {
            fromBlock: 0,
            toBlock: 100000,
            addresses: vec![],
            topics: vec![],
        };
        let address = ContractHistory::get_last_from_file(
            "./contracts/contracthistory.toml",
            bcos3client.get_full_name().as_str(),
            contractname,
        )
        .unwrap();
        event_sub_param.addresses.push(address.clone());
        let event = contractabi.find_event_by_name(methodname).unwrap();
        let eventsig = contractabi.event_abi_utils.event_signature(&event.clone());
        // 如果加上这行，就只监听onset事件
        //event_sub_param.topics.push(hex::encode(eventsig.as_bytes()));
        let paramstr = serde_json::to_string(&event_sub_param).unwrap();
        println!("event sub param : {:?}", event_sub_param);
        println!("event sub param(in string): {}", paramstr);

        if mode.as_str() == "async" {
            //给bcos3 c sdk传入一个独立的函数回调，不用wait模式，在生命周期中，监听的事件会一直出发回调函数
            //这里构造的一个上下文结构体，将其指针传给了c sdk，会带到回调函数里，要保证其实例一直没有被释放，即指针有效
            let event_context = EventCallbackContext {
                contractname: contractname.to_string(),
                contractpath: bcos3client.config.common.contractpath,
                hashtype: CommonHash::crypto_to_hashtype(&bcos3client.config.common.crypto),
            };
            bcos_event_sub_subscribe_event(
                bcos3client.sdk,
                str2p!(bcos3client.group.as_str()),
                str2p!(paramstr.as_str()),
                demo_bcos3_event_callback,
                &event_context as *const EventCallbackContext as *const c_void,
            );
            let mut n = 0;
            while n < 15 {
                thread::sleep(Duration::from_secs(1));
                if n % 3 == 0 {
                    println!("ticking {}", n);
                }
                n = n + 1;
            }
            return Ok(());
        } else {
            //采用wait模式，构建一个bcos3特有的future对象，采用mpsc channel实现同步轮询等待回调
            //(sdk: *const c_void, group: *const c_char, param: *const c_char, callback: BCOS3SDK_CALLBACK_FUNC, context: *const c_void) -> *const c_char;
            bcos_event_sub_subscribe_event(
                bcos3client.sdk,
                str2p!(bcos3client.group.as_str()),
                str2p!(paramstr.as_str()),
                Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
                Bcos3SDKFuture::to_c_ptr(&cbfuture),
            );
            let lasterr = bcos_sdk_get_last_error();
            if lasterr != 0 {
                let lasterrmsg = bcos_sdk_get_last_error_msg();
                let msgstr = CString::from_raw(lasterrmsg).to_str().unwrap().to_string();
                println!("event sub error {}", msgstr)
            }
            let mut n = 0;
            let mut count = 0;
            while n < 20 {
                //在此循环等待回调
                let res = cbfuture.wait_result();
                match res {
                    Ok(v) => {
                        count = count + 1;
                        println!("{} >--->(Sync) GOT Event callback", count);
                        println!("raw content {:?}", v);
                        if v.is_array() {
                            println!("->display logs :");
                            let logs = contractabi.parse_receipt_logs(&v)?;
                            display_receipt_logs(&logs);
                        } else {
                            println!("event sub got v : {:?}", v)
                        }
                        thread::sleep(Duration::from_secs(1));
                    }
                    Err(e) => {
                        match e.kind {
                            KissErrKind::ETimeout => {
                                //println!("Wait Result error {:?}", e);
                            }
                            _ => {
                                println!("wait error {:?}", e);
                            }
                        }
                    }
                }

                n = n + 1;
                if n > 10 {
                    break;
                }
            }
        }
        bcos3client.finish();

        Ok(())
    }
}
