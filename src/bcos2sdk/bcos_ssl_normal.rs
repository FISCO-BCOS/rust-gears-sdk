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
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

use openssl::ssl::{
    ShutdownResult, SslConnector, SslConnectorBuilder, SslFiletype, SslMethod, SslStream,
    SslVerifyMode,
};

use crate::bcos2sdk::bcos_channel_client::IBcosChannel;
use crate::bcos2sdk::channelpack::ChannelPack;
use crate::bcossdkutil::bcosclientconfig::ChannelConfig;
use crate::bcossdkutil::bufferqueue::BufferQueue;
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use crate::{kisserr, printlnex};
use std::io;

pub trait ISslStreamWrap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    fn shutdown(&mut self) -> Result<ShutdownResult, openssl::ssl::Error>;
}
//啥也不做，知识为了满足结构体初始化时要赋值的需求
struct EmptySslStreamWrap {}
impl ISslStreamWrap for EmptySslStreamWrap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(0 as usize)
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(0 as usize)
    }

    fn shutdown(&mut self) -> Result<ShutdownResult, openssl::ssl::Error> {
        Ok(ShutdownResult::Sent)
    }
}

struct SslStreamWrap {
    pub ssl_stream: SslStream<TcpStream>,
}
impl ISslStreamWrap for SslStreamWrap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        //println!("ISslStreamWrap write {:?}",buf.len());
        let res = self.ssl_stream.write(buf);
        match res {
            Ok(t) => return Ok(t),
            Err(e) => {
                let raw_code = e.raw_os_error().unwrap();
                //println!("SslStream send return {:?},",raw_code);
                match raw_code {
                    10035 => {
                        //10035表示would block @windows
                        return Ok(0);
                    }
                    11 => {
                        //11表示would block @linux
                        return Ok(0);
                    }
                    __ => return Err(e),
                }
            }
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        //println!("SslStreamWrap recv");
        let res = self.ssl_stream.read(buf);
        //println!("SslStreamWrap recv {:?}",res);
        match res {
            Ok(t) => return Ok(t),
            Err(e) => {
                let raw_code = e.raw_os_error().unwrap();
                //println!("SslStream read return {:?},", raw_code);
                match raw_code {
                    10035 => {
                        //10035表示would block
                        return Ok(0);
                    }
                    11 => {
                        //11表示would block @linux
                        return Ok(0);
                    }
                    __ => return Err(e),
                }
            }
        }
    }

    fn shutdown(&mut self) -> Result<ShutdownResult, openssl::ssl::Error> {
        self.ssl_stream.shutdown()
    }
}

///ssl 客户端，封装系统SSL的调用，只支持非国密ECDSA。
/// 要支持国密tls，参见bcos_nativetls_client.rs，这个文件里的实现同时支持gm和ecdsa
#[derive()]
pub struct BcosSSLClient {
    pub config: ChannelConfig,
    pub bufferqueue: BufferQueue,
    pub channelpackpool: Vec<ChannelPack>, //一个池子，存没有被处理的channelpack，在推送等流程用到

    pub is_valid: bool,
    pub is_connect: bool,
    //pub ssl_stream: Option<SslStream<TcpStream>>,
    pub sslstream_wrap: Box<dyn ISslStreamWrap + Send + Sync>,
}
//unsafe impl Send for  BcosSSLClient{}
//unsafe impl Sync for  BcosSSLClient{}
impl IBcosChannel for BcosSSLClient {
    fn finish(&mut self) {
        if self.is_valid != true {
            return;
        }
        //let stream = self.ssl_stream.take();
        //let res = stream.unwrap().shutdown();
        let res = self.sslstream_wrap.shutdown();
        self.is_valid = false;
        self.is_connect = false;
    }

    fn connect(&mut self) -> Result<i32, KissError> {
        let ctx = self.init()?;
        let connector = ctx.build();
        let res = connector
            .configure()
            .unwrap()
            .into_ssl(self.config.ip.as_str());
        let ssl = match res {
            Ok(s) => s,
            Err(e) => {
                return kisserr!(KissErrKind::ENetwork, "ssl_ctx into ssl {:?}", e);
            }
        };
        let tcp_stream =
            TcpStream::connect(format!("{}:{}", self.config.ip.as_str(), self.config.port))
                .unwrap();
        //let res = tcp_stream.set_nonblocking(true);
        let mut ssl_stream = match SslStream::new(ssl, tcp_stream) {
            Ok(s) => s,
            Err(e) => {
                return kisserr!(
                    KissErrKind::ENetwork,
                    "SslStream new from TcpStream error {:?}",
                    e
                );
            }
        };

        //ssl 握手连接
        let nio_res = ssl_stream.get_ref().set_nonblocking(false);
        let res = ssl_stream.connect();
        //println!("SSL Stream connect result {:?}", &res);
        match res {
            Ok(s) => (),
            Err(e) => {
                return kisserr!(KissErrKind::ENetwork, "SslStream connect error {:?}", e);
                //println!("connect error code :{:?}",e.code());
            }
        };

        //sslstream的connect倒不很需要异步，只是connect一下，在connect后再设置异步
        let nio_res = ssl_stream.get_ref().set_nonblocking(true);
        //self.ssl_stream = Option::from(ssl_stream);
        let sslstreamimpl = SslStreamWrap {
            ssl_stream: ssl_stream,
        };
        self.sslstream_wrap = Box::from(sslstreamimpl);
        //tcp_stream.set_nonblocking(true);
        Ok(0)
    }

    ///异步发送数据，如果未发送任何字节，返回0，可以重试发送
    fn send(&mut self, sendbuff: &Vec<u8>) -> Result<i32, KissError> {
        //take从option里借用出来一个stream实例，用完要还回去。否则下次再调用的时候这个option就是None了
        //看起来线程不安全了。
        //if let Some(mut stream) = self.ssl_stream.take() {
        if self.is_connect {
            //let res = stream.write(&sendbuff.as_slice());
            //self.ssl_stream = Option::from(stream);
            let res = self.sslstream_wrap.write(&sendbuff.as_slice());
            printlnex!("send res {:?}", res);
            match res {
                Ok(s) => return Ok(s as i32),
                Err(e) => return kisserr!(KissErrKind::ENetwork, "ssl send fail {:?}", e),
            }
        }
        return kisserr!(KissErrKind::ENetwork, "");
    }

    ///读取，c api要求输入一个预先分配好的缓冲区，讲读取的信息写入缓冲区带回
    fn recv(&mut self) -> Result<Vec<u8>, KissError> {
        let size = 10 * 1024;
        let mut recvbuffer: Vec<u8> = vec![0; size];
        printlnex!("recvbuffer size {}", recvbuffer.len());
        //take从option里借用出来一个stream实例，用完要还回去。否则下次再调用的时候这个option就是None了
        //看起来线程不安全了。
        //if let Some(mut stream) = self.ssl_stream.take() {
        if self.is_connect {
            // let res = stream.read(recvbuffer.as_mut_slice());
            // self.ssl_stream = Option::from(stream);
            let res = self.sslstream_wrap.read(recvbuffer.as_mut_slice());
            printlnex!("recv result {:?}", res);

            match res {
                Ok(size) => {
                    return Ok(recvbuffer[0..size].to_vec());
                }
                Err(e) => return kisserr!(KissErrKind::ENetwork, "ssl recv fail {:?}", e),
            };
        }
        return kisserr!(KissErrKind::ENetwork, "");
    }
}

//----------------------------------------
impl BcosSSLClient {
    pub fn default(config: &ChannelConfig) -> BcosSSLClient {
        BcosSSLClient {
            config: config.clone(),
            bufferqueue: BufferQueue::new(),
            is_valid: false,
            is_connect: false,
            channelpackpool: Vec::new(),
            //ssl_stream: Option::from(None),
            sslstream_wrap: Box::from(EmptySslStreamWrap {}),
        }
    }

    pub fn build(&mut self) -> Result<(), KissError> {
        //let  ctx = BcosSSLClient::init(&self.config)?;
        // let connector = ctx.build();
        self.connect()?;

        self.is_valid = true;
        self.is_connect = true;
        Ok(())
    }

    pub fn set_client_certs(
        ctx: &mut SslConnectorBuilder,
        config: &ChannelConfig,
    ) -> anyhow::Result<()> {
        ctx.set_ca_file(Path::new(config.cacert.as_str()))?;
        ctx.set_certificate_chain_file(Path::new(config.sdkcert.as_str()))?;
        ctx.set_private_key_file(Path::new(config.sdkkey.as_str()), SslFiletype::PEM)?;
        ctx.check_private_key()?;
        Ok(())
    }

    pub fn init(&mut self) -> Result<SslConnectorBuilder, KissError> {
        let mut ctx = match SslConnector::builder(SslMethod::tls_client()) {
            Ok(c) => c,
            Err(e) => {
                return kisserr!(KissErrKind::ENetwork, "sslconnector builder error {:?}", e);
            }
        };
        let curve = match openssl::ec::EcKey::from_curve_name(openssl::nid::Nid::SECP256K1) {
            Ok(c) => c,
            Err(e) => {
                return kisserr!(
                    KissErrKind::ENetwork,
                    "EcKey::from_curve_name error {:?}",
                    e
                );
            }
        };
        let res = match ctx.set_tmp_ecdh(&curve) {
            Ok(()) => (),
            Err(e) => {
                return kisserr!(KissErrKind::ENetwork, "sslconnector builder error {:?}", e);
            }
        };
        let res = match BcosSSLClient::set_client_certs(&mut ctx, &self.config) {
            Ok(()) => (),
            Err(e) => {
                return kisserr!(KissErrKind::ENetwork, "set client certs error {:?}", e);
            }
        };

        ctx.set_verify(SslVerifyMode::NONE);
        Ok(ctx)
    }
}
