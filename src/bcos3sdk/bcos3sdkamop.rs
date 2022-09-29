/*amop回调示例，未完整实现
参见： https://fisco-bcos-doc.readthedocs.io/zh_CN/latest/docs/develop/sdk/c_sdk/api.html#bcos-amop-subscribe-topic-with-cb
*/
use std::ffi::{c_char, CStr};

use crate::bcos3sdk::bcos3sdkresponse::{bcos_sdk_c_struct_response, Bcos3SDKResponse};

pub extern "C" fn amop_sub_callback(
    endpoint: *const c_char,
    seq: *const c_char,
    resp: *const bcos_sdk_c_struct_response,
) {
    unsafe {
        let mut endpointStr = "".to_string();
        let mut seqStr = "".to_string();
        if !endpoint.is_null() {
            endpointStr = CStr::from_ptr(endpoint.clone())
                .to_str()
                .unwrap()
                .to_string();
        }
        if !seq.is_null() {
            seqStr = CStr::from_ptr(seq.clone()).to_str().unwrap().to_string();
        }
        let response = Bcos3SDKResponse::from_callback(resp);
        //println!("endpoint {},seq {}", endpointStr, seqStr);
    }
}
