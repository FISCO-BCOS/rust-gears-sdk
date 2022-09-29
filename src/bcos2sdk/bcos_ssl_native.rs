/*
  FISCO BCOS/rust-SDK is a rust client for FISCO BCOS2.0 (https://github.com/FISCO-BCOS/)
  FISCO BCOS/rust-SDK is free software: you can redistribute it and/or modify it under the
  terms of the MIT License as published by the Free Software Foundation. This project is
  distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
  the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
  @author: kentzhang
  @date: 2021-07
*/
#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_variables,
    unused_assignments
)]
extern crate libloading;

use std::convert::From;
use std::{env, thread};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use libc::c_void;
use libloading::{Library, Symbol};
use serde_json::json;

use crate::bcos2sdk::bcos_channel_client::IBcosChannel;
use crate::bcos2sdk::bcosrpcwraper::RpcRequestData;
use crate::bcos2sdk::channelpack::{make_channel_pack, ChannelPack, CHANNEL_PACK_TYPE};
use crate::bcossdkutil::bcosclientconfig::{BcosCryptoKind, ChannelConfig, ClientConfig};
use crate::bcossdkutil::bufferqueue::BufferQueue;
use crate::bcossdkutil::commonutil::is_windows;
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use crate::{kisserr, printlnex};

//C native api 类型定义
type FN_SSOCK_CREATE = fn() -> *mut ::libc::c_void;
type FN_SSOCK_FINISH = fn(*mut ::libc::c_void);
type FN_SSOCK_RELEASE = fn(*mut ::libc::c_void);
type FN_SSOCK_INIT = fn(
    p: *mut ::libc::c_void,
    ca: *const u8,
    sdk: *const u8,
    key: *const u8,
    ensdk: *const u8,
    enkey: *const u8,
) -> i32;
type FN_SSOCK_SET_ECHO_MODE = fn(*mut ::libc::c_void, i32);
type FUNC_SSOCK_TRY_CONNECT = fn(*mut c_void, host: *const u8, port: i32) -> i32;
type FUNC_SSOCK_SEND = fn(*mut c_void, buffer: *const u8, len: i32) -> i32;
type FUNC_SSOCK_RECV = fn(*mut c_void, buffer: *mut u8, size: i32) -> i32;
type SSOCKLIB_TYPE = Arc<Mutex<*mut libc::c_void>>;

#[macro_export]
macro_rules! mac_pssock {
    ($x:expr) => {
        *$x.ssocklib
            .as_ref()
            .unwrap()
            .pssock
            .as_ref()
            .lock()
            .unwrap()
    };
}

pub fn lib_usage_msg() -> String {
    let msg: String = r###"
            此版本的tls链接依赖tassl动态库和c/c++原生代码实现，正常运行的注意点：
            1）所有符合当前操作系统的动态库齐备。
            2）动态库放于和目标运行程序相同的目录，或当前操作系统寻址加载动态库的目录。
            3）如有版本出入，可能需要重新编译tassl库和c/c++原生代码。
            具体参见“native_ssock_lib”的readme.md（本项目readme有连接）
            "###
    .to_string();

    msg
}

///强行在String的 最后补'\0',以作为c风格字符串传给c库，否则会有乱字符串传进去
fn rzero(s: &String) -> String {
    let mut n = s.clone();
    n.push('\0');
    n
}

#[derive(Debug)]
pub struct SSockNativeLib {
    pub pssock: SSOCKLIB_TYPE,
    //Arc<*mut libc::c_void>,
    pub nativelib: Library,
}

//unsafe impl Send for SSockNativeLib {}
//unsafe impl Sync for SSockNativeLib {}

impl SSockNativeLib {
    pub fn close(&mut self) {}
}

///tls 客户端，封装c动态库加载，c native api调用
/// 依赖GMSSL库，采用动态库连接的方式调用，参见上面的lib_usage_msg
#[derive(Debug)]
pub struct BcosNativeTlsClient {
    pub ssocklib: Option<SSockNativeLib>,
    pub config: ChannelConfig,
    pub bufferqueue: BufferQueue,
    pub is_valid: bool,
    pub is_connect: bool,
    pub channelpackpool: Vec<ChannelPack>, //一个池子，存没有被处理的channelpack，在推送等流程用到
}

unsafe impl Send for BcosNativeTlsClient {}

unsafe impl Sync for BcosNativeTlsClient {}

impl IBcosChannel for BcosNativeTlsClient {
    ///tls连接，会发起握手
    fn connect(&mut self) -> Result<i32, KissError> {
        unsafe {
            let func_connect: Symbol<FUNC_SSOCK_TRY_CONNECT> = self
                .ssocklib
                .as_ref()
                .unwrap()
                .nativelib
                .get(b"ssock_try_connect")
                .unwrap();
            let r = func_connect(
                mac_pssock!(self),
                rzero(&self.config.ip).as_ptr(),
                self.config.port as i32,
            );
            if r < 0 {
                return kisserr!(KissErrKind::ENetwork, "tls connect error {}", r);
            }
            Ok(r)
        }
    }

    ///异步发送数据，如果未发送任何字节，返回0，可以重试发送
    fn send(&mut self, sendbuff: &Vec<u8>) -> Result<i32, KissError> {
        unsafe {
            let func_send: Symbol<FUNC_SSOCK_SEND> = self
                .ssocklib
                .as_ref()
                .unwrap()
                .nativelib
                .get(b"ssock_send")
                .unwrap();
            let r = func_send(
                //self.ssocklib.as_ref().unwrap().pssock,
                //mac_pssock!(self),
                mac_pssock!(self),
                sendbuff.as_ptr(),
                sendbuff.len() as i32,
            );
            if r < 0 {
                return kisserr!(KissErrKind::ENetwork, "send err {}", r);
            }
            Ok(r)
        }
    }

    ///读取，c api要求输入一个预先分配好的缓冲区，讲读取的信息写入缓冲区带回
    fn recv(&mut self) -> Result<Vec<u8>, KissError> {
        unsafe {
            //printlnex!("self status {},{}",self.is_valid,self.is_connect);
            let func_recv: Symbol<FUNC_SSOCK_SEND> = self
                .ssocklib
                .as_ref()
                .unwrap()
                .nativelib
                .get(b"ssock_recv")
                .unwrap();
            let size: usize = 1024 * 10;
            let mut recvbuffer: Vec<u8> = Vec::with_capacity(size);
            let pdst = recvbuffer.as_mut_ptr();
            //let dstlen = 1024;
            let r = func_recv(
                //self.ssocklib.as_ref().unwrap().pssock,
                mac_pssock!(self),
                pdst,
                size as i32,
            );
            //println!("recv result {:?}", r);
            if r >= 0 {
                //println!("recv size :{}",r);

                recvbuffer.set_len(r as usize);

                //println!("buffer {:?}",recvbuffer);
                Ok(recvbuffer)
            } else {
                return kisserr!(KissErrKind::Error, "recv {}", r);
            }
        }
    }
    fn finish(&mut self) {
        self.release();
    }
}

impl BcosNativeTlsClient {
    pub fn default(config: &ChannelConfig) -> BcosNativeTlsClient {
        BcosNativeTlsClient {
            ssocklib: None,
            config: config.clone(),
            bufferqueue: BufferQueue::new(),
            is_valid: false,
            is_connect: false,
            channelpackpool: Vec::new(),
        }
    }

    pub fn build(&mut self) -> Result<bool, KissError> {
        let res = BcosNativeTlsClient::openlib();
        let nativelib = match res {
            Ok(lib) => lib,
            Err(e) => {
                print!("open lib error {:?}", e);
                return Err(e);
            }
        };

        unsafe {
            let func_create: Symbol<FN_SSOCK_CREATE> = (&nativelib).get(b"ssock_create").unwrap();
            let pssock_ptr = func_create();
            let lib = SSockNativeLib {
                pssock: Arc::new(Mutex::new(pssock_ptr)),
                nativelib: nativelib,
            };

            self.ssocklib = Option::from(lib);
            self.set_echo_mode(self.config.nativelib_echo_mode as i32);

            self.init()?;
            // printlnex!("debug tls init done");
            self.connect()?;
            // printlnex!("debug  connect done");
            self.is_valid = true;
            self.is_connect = true;
            Ok(self.is_valid)
        }
    }

    pub fn locate_lib_path() -> String {
        let mut exepath = env::current_exe().unwrap();
        //println!("curr exe path:{:?}",exefile.canonicalize().unwrap().as_path());
        exepath.pop();
        let mut libname: String = "native_tassl_sock_wrap".to_string();
        if is_windows() {
            libname = libname + ".dll";
        } else {
            libname = format!("lib{}.so", libname);
        }
        // let fullpath = exepath.join(Path::new(libname.as_str()));
        //let pathstr =  fullpath.as_os_str().to_str().unwrap();
        let pathstr = libname;
        return pathstr.to_string();
    }
    ///寻找相应路径下的动态库
    pub fn openlib() -> Result<Library, KissError> {
        unsafe {
            //let currpath = Path::new("../..");

            //println!("curr exe :{:?}", env::current_exe().unwrap());
            //println!("curr exe path:{:?}",exefile.as_path().to_str());
            //println!("curr dir: {:?}", currpath.canonicalize().unwrap());
            //let dllfile = currpath.join("native_tassl_sock_wrap.dll");

            let lib_fullpath = BcosNativeTlsClient::locate_lib_path();
            //println!("lib file : {:?}", lib_fullpath);
            let res = Library::new(lib_fullpath.as_str());
            //println!("open lib {},res {:?}",lib_fullpath,res);
            match res {
                Ok(lib) => {
                    return Ok(lib);
                }
                Err(e) => {
                    return kisserr!(
                        KissErrKind::Error,
                        "load lib error [{}],{:?}",
                        lib_fullpath,
                        e
                    );
                }
            }
        }
    }
    ///在堆上创建c底层的对象，类似 ssl_new分配了一个context，openssl也是同理创建了一个c指针对象保存了下来，
    /// todo：安全性，生命周期再做优化
    pub fn create(&self) {
        unsafe {
            let func_create: Symbol<FN_SSOCK_CREATE> = self
                .ssocklib
                .as_ref()
                .unwrap()
                .nativelib
                .get(b"ssock_create")
                .unwrap();
            //let pssock = func_create();
            // println!("{:?}",pssock);
        }
    }
    ///释放在堆上分配的内容
    pub fn release(&mut self) {
        if !self.is_valid {
            printlnex!("bcostls client is no valid");
            return;
        };
        unsafe {
            let func_release: Symbol<FN_SSOCK_RELEASE> = self
                .ssocklib
                .as_ref()
                .unwrap()
                .nativelib
                .get(b"ssock_release")
                .unwrap();
            func_release(mac_pssock!(self));
            //self.ssocklib.as_ref().unwrap().close();
            //self.ssocklib.as_ref().unwrap().pssock = ptr::null_mut();
            //self.ssocklib.unwrap().nativelib.close();
            self.ssocklib = Option::None;
            self.is_connect = false;
            self.is_valid = false;
            printlnex!("bcostls client is released!");
        }
    }

    ///设置底层api的打印级别，0，完全不打印，1打印，todo：在c底层支持日志
    pub fn set_echo_mode(&self, mode: i32) {
        // (self.nativelib.ssock_set_echo_mode)(self.ssocklib.as_ref().unwrap().pssock,mode);
        unsafe {
            let func_set: Symbol<FN_SSOCK_SET_ECHO_MODE> = self
                .ssocklib
                .as_ref()
                .unwrap()
                .nativelib
                .get(b"ssock_set_echo_mode")
                .unwrap();
            func_set(mac_pssock!(self), mode);
            //println!("set echo mode:{:?},mode {}", self.ssocklib.as_ref().unwrap().pssock,mode);
        }
    }

    ///初始化，根据tlskind配置的不同，可以同时支持国密和ecdsa的连接类型，传入不同的证书即可
    pub fn init(&self) -> Result<i32, KissError> {
        unsafe {
            let func_init: Symbol<FN_SSOCK_INIT> = self
                .ssocklib
                .as_ref()
                .unwrap()
                .nativelib
                .get(b"ssock_init")
                .unwrap();
            //printlnex!("debug get ssock_init done");
            //let r =func_init(self.ssocklib.as_ref().unwrap().pssock,self.cacrt.as_str().as_ptr(),self.sdkcrt.as_str().as_ptr(),self.sdkkey.as_str().as_ptr(),"","");
            let r;
            match self.config.tlskind {
                BcosCryptoKind::ECDSA => {
                    r = func_init(
                        mac_pssock!(self),
                        rzero(&self.config.cacert).as_ptr(),
                        rzero(&self.config.sdkcert).as_ptr(),
                        rzero(&self.config.sdkkey).as_ptr(),
                        rzero(&"".to_string()).as_ptr(),
                        rzero(&"".to_string()).as_ptr(),
                    );
                }
                BcosCryptoKind::GM => {
                    r = func_init(
                        mac_pssock!(self),
                        rzero(&self.config.gmcacert).as_ptr(),
                        rzero(&self.config.gmsdkcert).as_ptr(),
                        rzero(&self.config.gmsdkkey).as_ptr(),
                        rzero(&self.config.gmensdkcert).as_ptr(),
                        rzero(&self.config.gmensdkkey).as_ptr(),
                    );
                }
            }
            if r < 0 {
                return kisserr!(KissErrKind::ENetwork, "init tls client error {}", r);
            }
            Ok(r)
        }
    }
}

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

pub fn test_ssl_native() {
    let config = ClientConfig::load("gm/conf/config.toml").unwrap();
    let mut client = BcosNativeTlsClient::default(&config.channel);
    let res = client.build();
    println!("client build result {:?}", res);
    println!("Client sock is {:?}", client.ssocklib);
    //let res = client.connect();
    //println!("connect result = {:?}",res);
    let buffer = getNodeVersionPack().unwrap().pack();
    let sendres = client.send(&buffer);
    println!("send result  = {:?}", sendres);
    loop {
        //let dstlen = 1024;
        let recvres = client.recv();
        //println!("recv result {:?}", r);
        if recvres.is_ok() {
            let recvbuffer = recvres.unwrap().clone();

            if recvbuffer.len() > 0 {
                println!("{:?}", recvbuffer);
                let p = ChannelPack::unpack(&recvbuffer).unwrap();
                println!("pack: {}", p.detail());
                println!("data: {}", String::from_utf8(p.data).unwrap());

                break;

            }

        }
        thread::sleep(Duration::from_millis(300));
    }
    client.finish();
}
