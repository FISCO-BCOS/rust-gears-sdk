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

use crate::bcos2sdk::bcos_ssl_native::BcosNativeTlsClient;
use crate::bcos2sdk::bcos_ssl_normal::BcosSSLClient;
use crate::bcos2sdk::channelpack::{make_channel_pack, ChannelPack, CHANNEL_PACK_TYPE};
use crate::bcossdkutil::bcosclientconfig::{BcosCryptoKind, ChannelConfig};
use crate::bcossdkutil::bufferqueue::BufferQueue;
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use crate::{kisserr, printlnex};
use std::sync::{Arc, Mutex};
use std::time::Duration;

///用接口抽象国密和非国密SSL底层实现，
/// 底层只关注对SSL或GMSSL的API调用，暴露几个简单的接口
pub trait IBcosChannel {
    fn connect(&mut self) -> Result<i32, KissError>;
    fn send(&mut self, sendbuff: &Vec<u8>) -> Result<i32, KissError>;
    fn recv(&mut self) -> Result<Vec<u8>, KissError>;
    fn finish(&mut self);
}

type IBcosChannelImpl = Arc<Mutex<dyn IBcosChannel + Send + Sync>>;

/// 对channel协议的组装和解析，同步异步调用是共通的，封装在BcosChannelClient
pub struct BcosChannelClient {
    pub channelimpl: IBcosChannelImpl,
    pub config: ChannelConfig,
    pub bufferqueue: BufferQueue,
    pub channelpackpool: Vec<ChannelPack>, //一个池子，存没有被处理的channelpack，在推送等流程用到
}

//unsafe impl Send for BcosChannelClient {}
//unsafe impl Sync for BcosChannelClient {}

impl IBcosChannel for BcosChannelClient {
    fn connect(&mut self) -> Result<i32, KissError> {
        self.channelimpl.lock().unwrap().connect()
        //self.channelimpl.connect()
    }

    fn send(&mut self, sendbuff: &Vec<u8>) -> Result<i32, KissError> {
        self.channelimpl.lock().unwrap().send(sendbuff)
        //self.channelimpl.send(sendbuff)
    }

    fn recv(&mut self) -> Result<Vec<u8>, KissError> {
        //println!("recv");
        let res = self.channelimpl.lock().unwrap().recv();
        //println!("recv done");
        res
    }

    fn finish(&mut self) {
        self.channelimpl.lock().unwrap().finish()
    }
}

impl BcosChannelClient {
    pub fn default(config: &ChannelConfig) -> BcosChannelClient {
        let ssl_client = BcosSSLClient::default(&config);
        let channelimpl: IBcosChannelImpl = Arc::new(Mutex::new(ssl_client));
        //let channelimpl: IBcosChannelImpl = Arc::new(ssl_client);
        BcosChannelClient {
            config: config.clone(),
            bufferqueue: Default::default(),
            channelimpl: channelimpl,
            channelpackpool: vec![],
        }
    }
    pub fn new(config: &ChannelConfig) -> Result<BcosChannelClient, KissError> {
        let channelimpl: IBcosChannelImpl;
        println!("config.tlskind {:?}", &config);
        match config.tlskind {
            BcosCryptoKind::ECDSA => {
                let mut ssl_client = BcosSSLClient::default(&config);
                ssl_client.build()?;
                channelimpl = Arc::new(Mutex::new(ssl_client))
                //channelimpl = Arc::new(ssl_client)
            }
            BcosCryptoKind::GM => {
                let mut tls_client = BcosNativeTlsClient::default(&config);
                tls_client.build()?;
                channelimpl = Arc::new(Mutex::new(tls_client));
                //channelimpl = Arc::new(tls_client)
            }
        }

        let wraper = BcosChannelClient {
            config: config.clone(),
            bufferqueue: Default::default(),
            channelimpl: channelimpl,
            channelpackpool: vec![],
        };
        Ok(wraper)
    }

    ///尝试最多5次异步发送
    pub fn try_send(&mut self, outbuffer: &Vec<u8>) -> Result<i32, KissError> {
        let mut i: u32 = 0;
        while i < 5 {
            let res = self.send(outbuffer)?;
            if res > 0 {
                return Ok(res);
            }
            std::thread::sleep(Duration::from_millis(50));
            i += 1;
        }
        kisserr!(KissErrKind::ENetwork, "send none bytes after try")
    }
    /// 按配置的超时时间读socket
    pub fn try_recv(&mut self) -> Result<Vec<u8>, KissError> {
        let mut i = 0;
        let start = time::now();
        while time::now() - start < chrono::Duration::seconds(self.config.timeout as i64) {
            let res = self.recv()?;
            //println!(">> try recv {}", res.len());
            if res.len() > 0 {
                return Ok(res);
            }
            i += 1;
            std::thread::sleep(Duration::from_millis(100));
        }
        kisserr!(KissErrKind::ENetwork, "recv time out")
    }

    ///传入json 字符串，打入channelpack发送出去，同步等待read，然后返回从channelpack解析好的value
    pub fn request_sync(&mut self, reqtext: &str) -> Result<String, KissError> {
        let outpack = make_channel_pack(CHANNEL_PACK_TYPE::RPC, reqtext).unwrap();
        let returnpack = self.request_channelpack_sync(&outpack)?;
        let res = String::from_utf8(returnpack.data);
        match res {
            Ok(s) => {
                return Ok(s);
            }
            Err(e) => {
                return kisserr!(KissErrKind::ENetwork, "pack data is not string {:?}", e);
            }
        }
    }

    ///尝试从缓冲区获得一个对应的回包
    pub fn try_match_channelpack(
        &mut self,
        outpack: &ChannelPack,
    ) -> Result<ChannelPack, KissError> {
        //从缓冲区中match出一个回包，最多尝试50次
        let mut i = 0;
        let mut thepack: Option<ChannelPack> = Option::None;
        //println!("want pack type: 0x{:02X},seq: {}",outpack.packtype,outpack.seq);

        while i < 50 {
            // println!("before peek, queue size : {}",&self.bufferqueue.queue.len());
            if self.bufferqueue.queue.len() <= 42 {
                break;
            }
            let packres = ChannelPack::unpack(&self.bufferqueue.queue);
            match packres {
                Ok(pack) => {
                    //println!(">>>> get pack type: 0x{:02X},seq: {}",pack.packtype,pack.seq);
                    //从缓冲区中去掉已经解码的部分
                    self.bufferqueue.cut(pack.length);
                    //获得了一个回包，要判断下是否预期的回包，否则仅加入等待区

                    if pack.packtype == outpack.packtype && pack.seq == outpack.seq {
                        //println!(">>>match pack type: 0x{:02X},seq: {}",pack.packtype,pack.seq);
                        //是想要的回包
                        thepack = Option::from(pack);
                    } else {
                        //println!("not match  pack type: 0x{:02X},seq: {}",pack.packtype,pack.seq);
                        //是个完整的pack，但不是想要的，放入pool
                        self.channelpackpool.push(pack);
                    }
                }
                Err(e) => {
                    //从缓冲区中解码失败，通常原因是字节不够了，终止
                    //println!("no more data");
                    break;
                }
            }
            i += 1;
        }
        //返回发现的pack,或反馈一个空包错误
        match thepack {
            Some(pack) => Ok(pack),
            _ => {
                kisserr!(KissErrKind::EAgain, "no pack found")
            }
        }
    }

    pub fn pop_queue_to_packet(queue: &mut BufferQueue) -> Result<Vec<ChannelPack>, KissError> {
        let mut vecres: Vec<ChannelPack> = vec![];
        // println!("before peek, queue size : {}",&self.bufferqueue.queue.len());
        if queue.queue.len() <= 42 {
            return Ok(vecres);
        }
        let mut k = 0;
        loop {
            //println!("k = {}",k);
            if k > 50 {
                break;
            }
            k += 1;
            let packres = ChannelPack::unpack(&queue.queue);
            match packres {
                Ok(pack) => {
                    //println!(">>>> get pack type: 0x{:02X},seq: {}",pack.packtype,pack.seq);
                    //从缓冲区中去掉已经解码的部分
                    queue.cut(pack.length);
                    vecres.push(pack);
                }
                Err(e) => {
                    //从缓冲区中解码失败，通常原因是字节不够了，终止
                    //println!("no more data");
                    break;
                }
            }
        }
        Ok(vecres)
    }
    pub fn read_packets(&mut self) -> Result<Vec<ChannelPack>, KissError> {
        let mut i = 0;
        let mut vecres: Vec<ChannelPack> = vec![];
        //println!("try recv");
        while i < 1 {
            //println!("try recv-->");
            let mut res = self.recv()?;
            //println!("try recv res {:?}", res.len());
            if res.len() == 0 {
                break;
            }
            //读到的所有的数据先加入buffer
            self.bufferqueue.append(&mut res);
            vecres = BcosChannelClient::pop_queue_to_packet(&mut self.bufferqueue)?;
            i += 1
        }
        Ok(vecres)
    }

    pub fn read_to_match(&mut self, outpack: &ChannelPack) -> Result<ChannelPack, KissError> {
        let mut i = 0;
        while i < 50 {
            let mut res = self.try_recv()?;
            if res.len() == 0 {
                continue;
            }
            //读到的所有的数据先加入buffer
            self.bufferqueue.append(&mut res);
            let matchres = self.try_match_channelpack(outpack);
            match matchres {
                Ok(pack) => {
                    //获得了一个pack，返回即可
                    return Ok(pack);
                }
                _ => { /*继续等待即可*/ }
            }
            i += 1;
        }
        return kisserr!(KissErrKind::EAgain, "no data return");
    }
    ///这个方法主要是保证首先数据能发送出去（重试几次），然后从网络读入数据，直到得到发送的pack所对应的type和seq的回包
    pub fn request_channelpack_sync(
        &mut self,
        outpack: &ChannelPack,
    ) -> Result<ChannelPack, KissError> {
        let outbuffer = outpack.pack();
        printlnex!("chanel buffer length {} ", outbuffer.len());
        self.try_send(&outbuffer)?;
        self.read_to_match(&outpack)
    }
}
