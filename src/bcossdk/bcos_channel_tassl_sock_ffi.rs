#[cfg(feature = "tassl_sock_ffi")]
extern crate libc;

#[cfg(feature = "tassl_sock_ffi")]
use std::ffi::CStr;
#[cfg(feature = "tassl_sock_ffi")]
use std::ffi::CString;

#[cfg(feature = "tassl_sock_ffi")]
use libc::c_char;
#[cfg(feature = "tassl_sock_ffi")]
use libc::c_int;
#[cfg(feature = "tassl_sock_ffi")]
use libc::c_void;
#[cfg(feature = "tassl_sock_ffi")]
use serde_json::json;

#[cfg(feature = "tassl_sock_ffi")]
use crate::bcossdk::bcosrpcwraper::RpcRequestData;
#[cfg(feature = "tassl_sock_ffi")]
use crate::bcossdk::channelpack::{CHANNEL_PACK_TYPE, ChannelPack, make_channel_pack};

//use std::ffi::CStr;
//use libc::size_t;
//ffi 模式的调用，需要native_ssock_wrap.lib文件
//打开tassl_sock_ffi特性，需要用这个语句编译：cargo build --features  "tassl_sock_ffi"


#[cfg(feature = "tassl_sock_ffi")]
pub fn getNodeVersionPack() -> Option<ChannelPack>
{
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


#[cfg(feature = "tassl_sock_ffi")]
extern "C" fn fn_callback(buffer: *mut c_char, buffersize: c_int) -> c_int
{
    println!("IN CALLBACK {}", buffersize);
    println!("IN CALLBACK {:?}", buffer);
    unsafe {
        let mut cs = CStr::from_ptr(buffer.clone());
        println!("cs : {:?}", &cs);
        let content = "1024 from rust";
        buffer.copy_from(content.as_ptr() as *const c_char, content.len());

        return content.len() as c_int;
    }
}


#[cfg(feature = "tassl_sock_ffi")]
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

#[cfg(feature = "tassl_sock_ffi")]
pub fn test_ssock()
{
    test_callback();
    return;
    #[link(name = "native_tassl_sock_wrap")]
    extern "C" {
        fn ssock_create() -> *mut c_void;
        fn ssock_set_echo_mode(psock: *mut c_void, v: c_int);
        fn ssock_init(psock: *mut c_void,
                      ca_crt_file_: *const c_char,
                      sign_crt_file_: *const c_char,
                      sign_key_file_: *const c_char,
                      en_crt_file_: *const c_char,
                      en_key_file_: *const c_char,
        ) -> c_int;
        fn ssock_try_connect(psock: *mut c_void, ip: *const c_char, port: c_int) -> c_int;
        fn ssock_send(psock: *mut c_void, data: *const c_char, len: c_int) -> c_int;
        fn ssock_recv(psock: *mut c_void, data: *mut c_char, buffersize: c_int) -> c_int;
        fn ssock_finish(psock: *mut c_void);
    }

    unsafe {
        let cafile = CString::new("gm/sdk/gmca.crt").unwrap();
        let sdkcrt = CString::new("gm/sdk/gmsdk.crt").unwrap();
        let sdkkey = CString::new("gm/sdk/gmsdk.key").unwrap();
        let ensdk = CString::new("gm/sdk/gmensdk.crt").unwrap();
        let ensdkkey = CString::new("gm/sdk/gmensdk.key").unwrap();

        let psock = ssock_create();
        println!("{:?}", psock);
        ssock_set_echo_mode(psock, 1 as c_int);
        ssock_init(
            psock,
            cafile.as_ptr(),
            sdkcrt.as_ptr(),
            sdkkey.as_ptr(),
            ensdk.as_ptr(),
            ensdkkey.as_ptr(),
        );
        let ip = CString::new("119.29.114.153").unwrap();
        let res = ssock_try_connect(psock, ip.as_ptr(), 20800);
        println!("connnect result {}", res);
        let pack = getNodeVersionPack();
        let reqdata = pack.unwrap().pack();
        let res = ssock_send(psock, reqdata.as_ptr() as *mut c_char, reqdata.len() as c_int);
        let size: usize = 1024 * 10;
        let mut recvbuffer: Vec<u8> = Vec::with_capacity(size);
        let pdst = recvbuffer.as_mut_ptr();
        loop {
            //let dstlen = 1024;
            let r = ssock_recv(
                //self.ssocklib.as_ref().unwrap().pssock,
                psock,
                pdst as *mut c_char, size as i32);
            //println!("recv result {:?}", r);
            if r > 0 {
                //println!("recv size :{}",r);
                println!("r = {}", r);
                recvbuffer.set_len(r as usize);

                println!("{:?}", recvbuffer);
                let p = ChannelPack::unpack(&recvbuffer).unwrap();
                println!("pack: {}", p.detail());
                break;
            }
        }
        ssock_finish(psock);
    }
}


#[cfg(not(feature = "tassl_sock_ffi"))]
pub fn test_ssock() {
    println!("ffi not implement")
}