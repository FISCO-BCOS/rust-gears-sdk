/*
sdk回调的response结构体定义，以及rust语言的典型处理封装
参见https://fisco-bcos-doc.readthedocs.io/zh_CN/latest/docs/develop/sdk/c_sdk/api.html
*/
extern crate libc;

use std::ffi::CStr;

use libc::c_char;
use libc::c_void;
use serde_json::{Result as JsonResult, Value as JsonValue};

use crate::{kisserr, kisserrcode};
use crate::bcossdk::kisserror::{KissErrKind, KissError};

#[cfg(feature = "bcos3sdk_ffi")]
#[repr(C)]
#[derive(Debug)]
pub struct bcos_sdk_c_struct_response
{
    pub error: i32,
    // 返回状态, 0成功, 其他失败
    pub desc: *const c_char,
    // 失败时描述错误信息
    pub data: *const c_void,
    // 返回数据, error=0 时有效
    pub size: usize,
    // 返回数据大小, error=0 时有效
    pub context: *const c_void,  // 回调上下文,调用接口时传入的`context`参数
}

#[cfg(feature = "bcos3sdk_ffi")]
#[derive(Debug, Clone)]
pub struct Bcos3SDKResponse {
    pub error: i32,
    pub desc: String,
    pub data: String,
    pub size: usize,
    pub context_pointer: *const c_void,
}


#[cfg(feature = "bcos3sdk_ffi")]
impl Bcos3SDKResponse {
    pub fn display(&self)
    {
        //println!("data {}", self.data);
        if self.size > 0 {
            let v: JsonValue = serde_json::from_str(self.data.as_str()).unwrap();
            println!("data string is : ({}): {}", self.data.len(), serde_json::to_string(&v).unwrap());
        } else {
            println!("data is empty :[]");
        }
    }


    //将c sdk返回的指针数据，“安全的”转成rust的数据结构
    pub fn from_callback(c_sdk_response: *const bcos_sdk_c_struct_response) -> Bcos3SDKResponse {
        unsafe {
            let mut strdata = "".to_string();
            let mut strdesc = "".to_string();

            //println!("has response {} | {}",strdata,strdesc);
            let size = (*c_sdk_response).size;
            //println!(">> size  is {}", size);
            if !(*c_sdk_response).data.is_null() {
                let mut recvbuffer: Vec<u8> = Vec::with_capacity(size);
                ((*c_sdk_response).data as *const c_char).copy_to(recvbuffer.as_mut_ptr() as *mut c_char, size);
                recvbuffer.set_len(size);
                strdata = String::from_utf8(recvbuffer).unwrap();
            }

            if !(*c_sdk_response).desc.is_null() {
                strdesc = CStr::from_ptr((*c_sdk_response).desc.clone()).to_str().unwrap().to_string();
                //println!("desc is {:?}", strdesc);
            }

            //println!("has response {} | {}",strdata,strdesc);
            let response = Bcos3SDKResponse {
                desc: strdesc,
                data: strdata,
                error: (*c_sdk_response).error,
                size: (*c_sdk_response).size,
                context_pointer: (*c_sdk_response).context,
            };
            response
        }
    }


    // 从节点的返回json里解出 ["result"]或["error"] 段
    pub fn get_result(&self) -> Result<JsonValue, KissError>
    {
        if self.error != 0 {
            return kisserr!(KissErrKind::Error,"response {}",self.error);
        }
        let jsonresult: JsonResult<JsonValue> = serde_json::from_str(self.data.as_str());
        match jsonresult {
            Ok(jsonvalue) => {
                //println!("jsonvalue : {:?}",jsonvalue);
                let res_in_response = jsonvalue.get("result");
                match res_in_response {
                    None => {
                        let error_in_response = jsonvalue.get("error");
                        match error_in_response {
                            None => {
                                return Ok(jsonvalue);
                                //return kisserr!(KissErrKind::EFormat,"result json is empty");
                            }
                            Some(err_result) => {
                                println!("err_result {:?}", err_result);
                                let errcode = err_result["code"].as_i64().unwrap();
                                let errmsg = err_result["message"].as_str().unwrap();
                                return kisserrcode!(KissErrKind::Error,errcode,"{}",errmsg);
                            }
                        }
                    }
                    Some(v) => { return Ok(v.clone()); }
                }
            }
            Err(e) => { return kisserr!(KissErrKind::EFormat,"json format: {:?}",e); }
        }
    }
}

