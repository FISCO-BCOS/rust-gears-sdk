use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_long;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkamop::amop_sub_callback;
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkfuture::Bcos3SDKFuture;
use libc::c_char;
use libc::c_int;
use libc::c_void;
use libloading::{Library, Symbol};
use serde_json::{json, Value as JsonValue};

use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkwrapper::bcos3sdk_def::*;
use fisco_bcos_rust_gears_sdk::bcos3sdk::bcos3sdkwrapper::*;
use fisco_bcos_rust_gears_sdk::str2p;

use crate::bcossdkutil::abi_parser::ABIParser;
use crate::bcossdkutil::commonhash::HashType;
use crate::bcossdkutil::contractabi::ContractABI;
//Result as JsonResult
use crate::bcossdkutil::kisserror::KissError;
use crate::bcossdkutil::liteutils::datetime_str;

pub fn test_bcos3sdk_ffi_rpc() -> Result<i32, KissError> {
    unsafe {
        println!("---->test_bcos3sdk_ffi-----");
        let p_sdk = init_bcos3sdk_lib("bcos3sdklib/bcos3_sdk_config.ini");
        if p_sdk == 0 as *const c_void {
            return Ok(0);
        }
        let mut seq = 100;
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_group_list", "efg");
        cbfuture.display();
        bcos_rpc_get_group_list(
            p_sdk,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            //to_void_p!(context)
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait status {:?}", status);
        status.unwrap().display();

        let groupname = "group0";
        //------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);
        cbfuture.display();
        let cbfuture = Bcos3SDKFuture::create(seq, "abcd", "for test");
        bcos_rpc_get_group_info(
            p_sdk,
            //b"group0".as_ptr() as *const c_char,//str2c!(groupname),
            str2p!(groupname),
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait status {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_peers", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_peers(
            p_sdk,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        //println!("wait result  {:?}", status);
        status.unwrap().display();
        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_group_peers", "for test");
        cbfuture.display();
        //let arc_context = Arc::new(context);
        //let pcontext =
        bcos_rpc_get_group_peers(
            p_sdk,
            str2p!(groupname),
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();
        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture =
            Bcos3SDKFuture::create(seq, "bcos_rpc_get_total_transaction_count", "for test");
        cbfuture.display();
        //let arc_context = Arc::new(context);
        //let pcontext =
        bcos_rpc_get_total_transaction_count(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_system_config_by_key", "for test");
        cbfuture.display();
        //let arc_context = Arc::new(context);
        //let pcontext =
        let key = "";
        bcos_rpc_get_system_config_by_key(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            str2p!(key),
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_consensus_status", "for test");
        cbfuture.display();
        //let arc_context = Arc::new(context);
        //let pcontext =
        bcos_rpc_get_consensus_status(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();
        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_sync_status", "for test");
        cbfuture.display();
        //let arc_context = Arc::new(context);
        //let pcontext =
        bcos_rpc_get_sync_status(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);
        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_pending_tx_size", "for test");
        cbfuture.display();
        //let arc_context = Arc::new(context);
        //let pcontext =
        bcos_rpc_get_pending_tx_size(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_pbft_view", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_pbft_view(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_observer_list", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_observer_list(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_sealer_list", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_sealer_list(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_code", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_code(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            "0xc0a367cb5d11f21fd51196e9683cbfc2b8cd33c2e86c559d67142152f5fa7ee5".as_ptr()
                as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_block_number", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_block_number(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_block_limit", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        let limit = bcos_rpc_get_block_limit(p_sdk, str2p!(groupname));
        println!("limit is  {}", limit);

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_block_hash_by_number", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_block_hash_by_number(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            1,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let response = cbfuture.wait().unwrap();
        let blockhash = String::from(response.get_result().unwrap().clone().as_str().unwrap());
        println!("BLOCK HASH is  {:?}", &blockhash);

        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_block_by_hash", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_block_by_hash(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            str2p!(blockhash.as_str()),
            0,
            0,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let result = cbfuture.wait().unwrap().get_result();
        println!(">>>>>>wait result  {:?}", result);
        if true {
            //return Ok(0);
        }

        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);
        let txhash = "0xf38851e1aecf890f7f978431dc61c9116f7565f5aab16f17353b3cb4e49853ca";
        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_transaction_receipt", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_rpc_get_transaction_receipt(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            str2p!(txhash),
            0,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_rpc_get_transaction", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        let txhash = "0xf38851e1aecf890f7f978431dc61c9116f7565f5aab16f17353b3cb4e49853ca";
        cbfuture.display();
        bcos_rpc_get_transaction(
            p_sdk,
            str2p!(groupname),
            0 as *const c_char,
            str2p!(txhash),
            0,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: {}", seq);

        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_event_sub_subscribe_event", "for test");
        //let arc_context = Arc::new(context);
        //let pcontext =
        cbfuture.display();
        bcos_event_sub_subscribe_event(
            p_sdk,
            str2p!(groupname),
            "param".as_ptr() as *const c_char,
            Bcos3SDKFuture::bcos_callback as BCOS3SDK_CALLBACK_FUNC,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        let status = cbfuture.wait();
        println!("wait result  {:?}", status);
        status.unwrap().display();

        bcos_event_sub_unsubscribe_event(p_sdk, "param".as_ptr() as *const c_char);

        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: ,bcos_amop_subscribe_topic{}", seq);
        let guangzhou = "guangzhou003";
        let shenzhen = "shenzhen0755";
        let topics = vec![str2p!(guangzhou), str2p!(shenzhen)];
        bcos_amop_subscribe_topic(p_sdk, topics.as_ptr(), 2);
        println!("bcos_amop_subscribe_topic done");
        //--------------------------------------------------------------------------------------
        seq = Bcos3SDKFuture::next_seq();
        println!("----------- start seq: ,bcos_amop_subscribe_topic{}", seq);
        let cbfuture = Bcos3SDKFuture::create(seq, "bcos_amop_subscribe_topic_with_cb", "for test");
        let topic = CString::new("cbtest001").unwrap();
        bcos_amop_subscribe_topic_with_cb(
            p_sdk,
            topic.as_ptr(),
            amop_sub_callback,
            Bcos3SDKFuture::to_c_ptr(&cbfuture),
        );
        println!("bcos_amop_subscribe_topic_with_cb done");

        println!("ready to quit");
        thread::sleep(Duration::from_secs(2));

        bcos_sdk_stop(p_sdk);
        bcos_sdk_destroy(p_sdk);
        println!("Ok to quit");
    }
    Ok(0)
}

pub fn test_bcos3sdk() {
    //test_bcos3sdk_ffi();
    let result = test_bcos3sdk_ffi_rpc();
    print!("test result {:?}", result);
}

//用直接打开library的方式，代码比较繁琐，要一个一个get，还是用ffi，直接映射symbol，这段library代码只是留在这里供参考
type FN_BCOS_SDK_CREATE = fn(*const c_char) -> *mut ::libc::c_void;
pub fn test_native_lib() {
    unsafe {
        println!("---->test_native_lib-----");
        let lib_fullpath = "./libbcos-c-sdk.dll".to_string();
        let res = Library::new(lib_fullpath.as_str());
        println!("{:?}", &res);
        let nativelib = res.unwrap();
        let loadres = (&nativelib).get(b"bcos_sdk_create_by_config");
        println!("{:?}", &loadres);
        let func_create: Symbol<FN_BCOS_SDK_CREATE> = loadres.unwrap();
        println!("{:?}", func_create);
    }
}
