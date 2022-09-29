#[cfg(feature = "bcos2sdk_ffi")]
use crate::bcos2sdk::bcos2_ssl_ffi::*;
#[cfg(feature = "bcos2sdk_ffi")]
use crate::bcos2sdk::bcosrpcwraper::RpcRequestData;
#[cfg(feature = "bcos2sdk_ffi")]
use crate::bcos2sdk::channelpack::{make_channel_pack, ChannelPack, CHANNEL_PACK_TYPE};
#[cfg(feature = "bcos2sdk_ffi")]
use crate::str2p;
#[cfg(feature = "bcos2sdk_ffi")]
use libc::{c_char, c_int};
#[cfg(feature = "bcos2sdk_ffi")]
use serde_json::json;
#[cfg(feature = "bcos2sdk_ffi")]
use std::ffi::CString;
#[cfg(feature = "bcos2sdk_ffi")]
use std::thread;
#[cfg(feature = "bcos2sdk_ffi")]
use std::time::Duration;

//use std::ffi::CStr;
//use libc::size_t;
//ffi 模式的调用，需要native_ssock_wrap.lib文件
//打开tassl_sock_ffi特性，需要用这个语句编译：cargo build --features  "tassl_sock_ffi"

#[cfg(feature = "bcos2sdk_ffi")]
pub fn getNodeVersionPack() -> Option<ChannelPack> {
    let groupid = 1;
    let cmd = "getClientVersion";
    let params_value = json!([groupid]);

    let req = RpcRequestData {
        method: cmd.to_string(),
        params: params_value.clone(),
        jsonrpc: "2.0".to_string(),
        id: 1,
    };
    println!("{:?}", req);
    make_channel_pack(CHANNEL_PACK_TYPE::RPC, req.encode().unwrap().as_str())
}

#[cfg(feature = "libtestcallback")]
extern "C" fn fn_callback(buffer: *mut c_char, buffersize: c_int) -> c_int {
    println!("IN CALLBACK {}", buffersize);
    println!("IN CALLBACK {:?}", buffer);
    unsafe {
        let cs = CStr::from_ptr(buffer.clone());
        println!("cs : {:?}", &cs);
        let content = "1024 from rust";
        buffer.copy_from(content.as_ptr() as *const c_char, content.len());

        return content.len() as c_int;
    }
}

#[cfg(feature = "libtestcallback")]
pub fn test_callback() {
    #[link(name = "libtestcallback")]
    extern "C" {
        fn dotest(fncb: *mut c_void);
    }
    unsafe {
        dotest(fn_callback as *mut c_void);
        {
            thread::sleep(Duration::from_secs(5));
        }
    }
}

pub fn test_ssock() {
    println!("test ssock ffi");
    #[cfg(feature = "bcos2sdk_ffi")]
    unsafe {
        /* let cafile = CString::new("gm/sdk/gmca.crt").unwrap();
        let sdkcrt = CString::new("gm/sdk/gmsdk.crt").unwrap();
        let sdkkey = CString::new("gm/sdk/gmsdk.key").unwrap();
        let ensdk = CString::new("gm/sdk/gmensdk.crt").unwrap();
        let ensdkkey = CString::new("gm/sdk/gmensdk.key").unwrap();*/
        let cafile = "gm/sdk/gmca.crt";
        let sdkcrt = "gm/sdk/gmsdk.crt";
        let sdkkey = "gm/sdk/gmsdk.key";
        let ensdk = "gm/sdk/gmensdk.crt";
        let ensdkkey = "gm/sdk/gmensdk.key";

        let psock = ssock_create();
        println!("{:?}", psock);
        ssock_set_echo_mode(psock, 1 as c_int);
        ssock_init(
            psock,
            //cafile.as_ptr(),
            str2p!(cafile),
            str2p!(sdkcrt),
            str2p!(sdkkey),
            str2p!(ensdk),
            str2p!(ensdkkey),
        );
        let ip = CString::new("119.29.114.153").unwrap();
        let res = ssock_try_connect(psock, ip.as_ptr(), 20800);
        println!("connnect result {}", res);
        let pack = getNodeVersionPack();
        let reqdata = pack.unwrap().pack();
        let res = ssock_send(
            psock,
            reqdata.as_ptr() as *mut c_char,
            reqdata.len() as c_int,
        );
        let size: usize = 1024 * 10;
        let mut recvbuffer: Vec<u8> = Vec::with_capacity(size);
        let pdst = recvbuffer.as_mut_ptr();
        loop {
            //let dstlen = 1024;
            let r = ssock_recv(
                //self.ssocklib.as_ref().unwrap().pssock,
                psock,
                pdst as *mut c_char,
                size as i32,
            );
            //println!("recv result {:?}", r);
            if r > 0 {
                //println!("recv size :{}",r);
                println!("r = {}", r);
                recvbuffer.set_len(r as usize);

                println!("{:?}", recvbuffer);
                let p = ChannelPack::unpack(&recvbuffer).unwrap();
                println!("pack(FFI): {}", p.detail());
                break;
            } else {
                thread::sleep(Duration::from_millis(100));
            }
        }
        ssock_finish(psock);
    }
}
