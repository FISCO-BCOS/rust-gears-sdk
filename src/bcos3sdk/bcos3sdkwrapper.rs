/*FFI方式封装C语言SDK接口，定义基本的数据结构和回调方法类型

接口文档参见:https://fisco-bcos-doc.readthedocs.io/zh_CN/latest/docs/develop/sdk/c_sdk/api.html
C语言sdk封装参见：https://github.com/FISCO-BCOS/bcos-c-sdk
CPP客户端实现参见：https://github.com/FISCO-BCOS/bcos-cpp-sdk

 */
extern crate libc;

//use std::ffi::CStr;

use libc::{c_char, c_void};
use std::ffi::CString;

//use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};

use crate::bcos3sdk::bcos3sdkresponse::bcos_sdk_c_struct_response;

use crate::str2p;

pub type BCOS3SDK_CALLBACK_FUNC = extern "C" fn(resp: *const bcos_sdk_c_struct_response);

pub type BCOS3SDK_AMOP_SUB_CALLBACK_FUNC = extern "C" fn(
    endpoint: *const c_char,
    seq: *const c_char,
    resp: *const bcos_sdk_c_struct_response,
);

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct EventSubParam {
    pub fromBlock: u64,
    pub toBlock: u64,
    pub addresses: Vec<String>,
    pub topics: Vec<String>,
}

//ffi方式链接bcos-c-sdk库且映射C API
#[cfg(feature = "bcos3sdk_ffi")]
pub mod bcos3sdk_def {

    use crate::bcos3sdk::bcos3sdkwrapper::{
        BCOS3SDK_AMOP_SUB_CALLBACK_FUNC, BCOS3SDK_CALLBACK_FUNC,
    };
    use libc::{c_char, c_int, c_ulong};
    use std::ffi::{c_longlong, c_void};

    #[link(name = "bcos-c-sdk")]
    extern "C" {
        pub fn bcos_sdk_version() -> *const c_char;
        pub fn bcos_sdk_create_by_config_file(configfile: *const c_char) -> *const c_void;
        pub fn bcos_sdk_start(sdk: *const c_void);
        pub fn bcos_sdk_stop(sdk: *const c_void);
        pub fn bcos_sdk_destroy(sdk: *const c_void);
        pub fn bcos_sdk_get_last_error_msg() -> *mut c_char;
        pub fn bcos_sdk_is_last_opr_success() -> c_int;
        pub fn bcos_sdk_get_last_error() -> c_int;
        pub fn bcos_rpc_get_group_info(
            sdk: *const c_void,
            group: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_group_list(
            sdk: *const c_void,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_group_info_list(
            sdk: *const c_void,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_group_node_info(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_transaction(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            tx_hash: *const c_char,
            proof: c_int,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_transaction_receipt(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            tx_hash: *const c_char,
            proof: c_int,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_block_by_hash(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            block_hash: *const c_char,
            only_header: c_int,
            only_tx_hash: c_int,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_block_by_number(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            block_number: c_longlong,
            only_header: c_int,
            only_tx_hash: c_int,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_block_hash_by_number(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            block_number: c_longlong,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_block_limit(sdk: *const c_void, group: *const c_char) -> c_longlong;
        pub fn bcos_rpc_get_block_number(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_code(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            address: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_sealer_list(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_observer_list(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_pbft_view(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_pending_tx_size(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_sync_status(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_consensus_status(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_system_config_by_key(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            key: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_total_transaction_count(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_group_peers(
            sdk: *const c_void,
            group: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        pub fn bcos_rpc_get_peers(
            sdk: *const c_void,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        //Event类接口
        pub fn bcos_event_sub_subscribe_event(
            sdk: *const c_void,
            group: *const c_char,
            param: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        ) -> *const c_char;
        pub fn bcos_event_sub_unsubscribe_event(sdk: *const c_void, group: *const c_char);

        //AMOP类sdk
        //void bcos_amop_subscribe_topic(void* sdk, char** topics, size_t count)
        pub fn bcos_amop_subscribe_topic(
            sdk: *const c_void,
            topics: *const *const c_char,
            count: c_ulong,
        );
        // void bcos_amop_subscribe_topic_with_cb(void* sdk, const char* topic, bcos_sdk_c_amop_subscribe_cb cb, void* context)
        pub fn bcos_amop_subscribe_topic_with_cb(
            sdk: *const c_void,
            topic: *const c_char,
            callback: BCOS3SDK_AMOP_SUB_CALLBACK_FUNC,
            context: *const c_void,
        );

        // void* bcos_sdk_create_keypair(int crypto_type); 1: ecdsa 2: sm
        pub fn bcos_sdk_create_keypair(crypto_type: c_int) -> *const c_void;

        // void* bcos_sdk_create_keypair_by_private_key(int crypto_type, const char* private_key)
        pub fn bcos_sdk_create_keypair_by_private_key(
            crypto_type: c_int,
            private_key: *const c_char,
        );

        // void* bcos_sdk_create_keypair_by_hex_private_key(int crypto_type, const char* private_key)
        pub fn bcos_sdk_create_keypair_by_hex_private_key(
            crypto_type: c_int,
            private_key: *const c_char,
        ) -> *const c_void;

        //int bcos_sdk_get_keypair_type(void* key_pair)
        pub fn bcos_sdk_get_keypair_type(private_key: *const c_char);

        pub fn bcos_sdk_get_keypair_public_key(key_pair: *const c_void);
        pub fn bcos_sdk_get_keypair_private_key(key_pair: *const c_void);
        pub fn bcos_sdk_get_group_chain_id(
            sdk: *const c_void,
            group: *const c_char,
        ) -> *const c_char;
        //void bcos_sdk_get_group_wasm_and_crypto(void* sdk, const char* group, int* wasm, int* sm_crypto);
        pub fn bcos_sdk_get_group_wasm_and_crypto(
            sdk: *const c_void,
            group: *const c_char,
            wasm: *mut c_int,
            sm_cryto: *mut c_int,
        );

        //sendTransaction
        //void bcos_rpc_send_transaction(void* sdk, const char* group, const char* node, const char* data,
        //int proof, bcos_sdk_c_struct_response_cb callback, void* context);
        pub fn bcos_rpc_send_transaction(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            data: *const c_char,
            proof: c_int,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );
        //void bcos_rpc_call(void* sdk, const char* group, const char* node, const char* to, const char* data,
        //     bcos_sdk_c_struct_response_cb callback, void* context)
        pub fn bcos_rpc_call(
            sdk: *const c_void,
            group: *const c_char,
            node: *const c_char,
            to: *const c_char,
            data: *const c_char,
            callback: BCOS3SDK_CALLBACK_FUNC,
            context: *const c_void,
        );

        // void bcos_sdk_create_signed_transaction
        // (void* key_pair, const char* group_id, const char* chain_id,
        // const char* to, const char* data, const char* abi, int64_t block_limit, int32_t attribute,
        // char** tx_hash, char** signed_tx)
        // 创建签名的交易,BCOS3用了新的交易编码方式，组装交易数据（含ABI格式的function input）需要用BCOS3的CAPI，ABI部分则通用兼容。
        pub fn bcos_sdk_create_signed_transaction(
            key_pair: *const c_void,
            group_id: *const c_char,
            chain_id: *const c_char,
            to: *const c_char,
            data: *const c_char,
            abi: *const c_char,
            blocklimit: c_longlong,
            attribute: c_int,
            tx_hash: *mut *mut c_char,
            signed_hash: *mut *mut c_char,
        );
        //内存管理
        pub fn bcos_sdk_c_free(p: *const c_void);
    }
}


//----------------------------------------------------------
//当未定义bcos3sdk_ffi时，声明一些unsafe的C语言库接口的“桩”方法，"骗过编"译器，实际上等于没有链接库，什么都做不了
#[cfg(not(feature = "bcos3sdk_ffi"))]
pub mod bcos3sdk_def {
    use crate::bcos3sdk::bcos3sdkwrapper::{
        BCOS3SDK_AMOP_SUB_CALLBACK_FUNC, BCOS3SDK_CALLBACK_FUNC,
    };
    use libc::{c_char, c_int, c_ulong};
    use std::ffi::{c_longlong, c_void};

    pub unsafe fn bcos_sdk_version() -> *const c_char {
        return 0 as *const c_char;
    }
    pub unsafe fn bcos_sdk_create_by_config_file(configfile: *const c_char) -> *const c_void {
        return 0 as *const c_void;
    }
    pub unsafe fn bcos_sdk_start(sdk: *const c_void) {}
    pub unsafe fn bcos_sdk_stop(sdk: *const c_void) {}
    pub unsafe fn bcos_sdk_destroy(sdk: *const c_void) {}
    pub unsafe fn bcos_sdk_get_last_error_msg() -> *mut c_char {
        return 0 as *mut c_char;
    }
    pub unsafe fn bcos_sdk_is_last_opr_success() -> c_int {
        return 0;
    }
    pub unsafe fn bcos_sdk_get_last_error() -> c_int {
        return -32999; // this errno for no implement
    }
    pub unsafe fn bcos_rpc_get_group_info(
        sdk: *const c_void,
        group: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_group_list(
        sdk: *const c_void,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub fn bcos_rpc_get_group_info_list(
        sdk: *const c_void,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_group_node_info(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_transaction(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        tx_hash: *const c_char,
        proof: c_int,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_transaction_receipt(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        tx_hash: *const c_char,
        proof: c_int,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_block_by_hash(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        block_hash: *const c_char,
        only_header: c_int,
        only_tx_hash: c_int,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_block_by_number(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        block_number: c_longlong,
        only_header: c_int,
        only_tx_hash: c_int,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_block_hash_by_number(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        block_number: c_longlong,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_block_limit(sdk: *const c_void, group: *const c_char) -> c_longlong {
        return 0 as c_longlong;
    }
    pub unsafe fn bcos_rpc_get_block_number(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_code(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        address: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_sealer_list(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_observer_list(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_pbft_view(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_pending_tx_size(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_sync_status(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_consensus_status(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_system_config_by_key(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        key: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_total_transaction_count(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_group_peers(
        sdk: *const c_void,
        group: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    pub unsafe fn bcos_rpc_get_peers(
        sdk: *const c_void,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    //Event类接口
    pub unsafe fn bcos_event_sub_subscribe_event(
        sdk: *const c_void,
        group: *const c_char,
        param: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) -> *const c_char {
        return 0 as *const c_char;
    }
    pub unsafe fn bcos_event_sub_unsubscribe_event(sdk: *const c_void, group: *const c_char) {}

    //AMOP类sdk
    //void bcos_amop_subscribe_topic(void* sdk, char** topics, size_t count)
    pub unsafe fn bcos_amop_subscribe_topic(
        sdk: *const c_void,
        topics: *const *const c_char,
        count: c_ulong,
    ) {
    }
    // void bcos_amop_subscribe_topic_with_cb(void* sdk, const char* topic, bcos_sdk_c_amop_subscribe_cb cb, void* context)
    pub unsafe fn bcos_amop_subscribe_topic_with_cb(
        sdk: *const c_void,
        topic: *const c_char,
        callback: BCOS3SDK_AMOP_SUB_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }

    // void* bcos_sdk_create_keypair(int crypto_type); 1: ecdsa 2: sm
    pub unsafe fn bcos_sdk_create_keypair(crypto_type: c_int) -> *const c_void {
        return 0 as *const c_void;
    }

    // void* bcos_sdk_create_keypair_by_private_key(int crypto_type, const char* private_key)
    pub unsafe fn bcos_sdk_create_keypair_by_private_key(
        crypto_type: c_int,
        private_key: *const c_char,
    ) {
    }

    // void* bcos_sdk_create_keypair_by_hex_private_key(int crypto_type, const char* private_key)
    pub unsafe fn bcos_sdk_create_keypair_by_hex_private_key(
        crypto_type: c_int,
        private_key: *const c_char,
    ) -> *const c_void {
        return 0 as *const c_void;
    }

    //int bcos_sdk_get_keypair_type(void* key_pair)
    pub unsafe fn bcos_sdk_get_keypair_type(private_key: *const c_char) {}

    pub unsafe fn bcos_sdk_get_keypair_public_key(key_pair: *const c_void) {}
    pub unsafe fn bcos_sdk_get_keypair_private_key(key_pair: *const c_void) {}
    pub unsafe fn bcos_sdk_get_group_chain_id(
        sdk: *const c_void,
        group: *const c_char,
    ) -> *const c_char {
        return 0 as *const c_char;
    }
    //void bcos_sdk_get_group_wasm_and_crypto(void* sdk, const char* group, int* wasm, int* sm_crypto);
    pub unsafe fn bcos_sdk_get_group_wasm_and_crypto(
        sdk: *const c_void,
        group: *const c_char,
        wasm: *mut c_int,
        sm_cryto: *mut c_int,
    ) {
    }

    //sendTransaction
    //void bcos_rpc_send_transaction(void* sdk, const char* group, const char* node, const char* data,
    //int proof, bcos_sdk_c_struct_response_cb callback, void* context);
    pub unsafe fn bcos_rpc_send_transaction(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        data: *const c_char,
        proof: c_int,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }
    //void bcos_rpc_call(void* sdk, const char* group, const char* node, const char* to, const char* data,
    //     bcos_sdk_c_struct_response_cb callback, void* context)
    pub unsafe fn bcos_rpc_call(
        sdk: *const c_void,
        group: *const c_char,
        node: *const c_char,
        to: *const c_char,
        data: *const c_char,
        callback: BCOS3SDK_CALLBACK_FUNC,
        context: *const c_void,
    ) {
    }

    // void bcos_sdk_create_signed_transaction
    // (void* key_pair, const char* group_id, const char* chain_id,
    // const char* to, const char* data, const char* abi, int64_t block_limit, int32_t attribute,
    // char** tx_hash, char** signed_tx)
    // 创建签名的交易,BCOS3用了新的交易编码方式，组装交易数据（含ABI格式的function input）需要用BCOS3的CAPI，ABI部分则通用兼容。
    pub unsafe fn bcos_sdk_create_signed_transaction(
        key_pair: *const c_void,
        group_id: *const c_char,
        chain_id: *const c_char,
        to: *const c_char,
        data: *const c_char,
        abi: *const c_char,
        blocklimit: c_longlong,
        attribute: c_int,
        tx_hash: *mut *mut c_char,
        signed_hash: *mut *mut c_char,
    ) {
    }
    //内存管理
    pub unsafe fn bcos_sdk_c_free(p: *const c_void) {}
}

use crate::bcos3sdk::bcos3sdkwrapper::bcos3sdk_def::{
    bcos_sdk_create_by_config_file, bcos_sdk_start,
};

pub unsafe fn init_bcos3sdk_lib(sdk_cfgfile: &str) -> *const c_void {
    unsafe {
        let sdk = bcos_sdk_create_by_config_file(str2p!(sdk_cfgfile));
        if sdk == 0 as *const c_void {
            return sdk;
        }
        bcos_sdk_start(sdk);
        return sdk;
    }
}
