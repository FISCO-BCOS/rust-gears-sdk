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

//extern crate bincode;

use std::convert::From;
use std::convert::TryInto;

use ethereum_types::H256;
use keccak_hash::keccak;

use crate::bcossdk::kisserror::{KissErrKind, KissError};
#[derive(Debug, Clone, Eq, PartialEq,Hash)]
pub enum CHANNEL_PACK_TYPE{
    RPC = 0x12,	//JSONRPC 2.0格式	RPC接口消息包	SDK->节点
    HEART_BEAT = 0x13,	//json格式心跳包{"heartbeat":"0"}	心跳包	0:SDK->节点，1:节点->SDK
    HAND_SHAKE = 0x14,	//SDK->节点的包体{"minimumSupport":version,"maximumSupport":version,"clientType":"client type"},节点->SDK的包体{"protocol":version,"nodeVersion":"fisco-bcos version"	握手包，json格式的协议版本协商	SDK<->节点，双向
    CLIENT_REGISTER_EVENT_LOG = 0x15 ,// 注册消息监听
    AMOP_REQ = 0x30,	//AMOP消息包包体	AMOP请求包	SDK<->节点，双向
    AMOP_RESP = 0x31,	//失败的AMOP消息的包体	AMOP失败响应包	节点->SDK或节点->节点
    TOPIC_REPORT = 0x32,	//json数组，存储SDK监听的Topics	上报Topic信息	SDK->节点
    TOPIC_MULTICAST = 0x35,	//AMOP消息包包体	AMOP多播消息	节点->节点
    TX_COMMITTED = 0x1000,	//json格式的交易上链通知	交易上链回调	节点->SDK
    TX_BLOCKNUM = 0x1001,	//json格式的区块上链通知{"groupID":"groupID","blockNumber":"blockNumber"}
    EVENT_LOG_PUSH = 0x1002
}

///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/design/protocol_description.html#channelmessage-v2
///
///length	uint32_t	4	数据包长度，含包头和数据，大端
///
///type	uint16_t	2	数据包类型，大端
///
///seq	string	32	数据包序列号，32字节uuid
///
///result	int32_t	4	错误码，大端
///
///data	bytes	length-42	数据包体，字节流
#[derive(Default, Clone, Debug)]
pub struct ChannelPack {
    pub length: usize,
    pub packtype: u16,
    pub seq: H256,
    pub result: u32,
    pub data: Vec<u8>,
}

impl ChannelPack {
    pub fn detail(&self) -> String {
        format!(
            "len:{},type:0x{:X},seq:{:?},result:{},data:{}",
            self.length,
            self.packtype,
            self.seq,
            self.result,
            String::from_utf8(self.data.clone()).unwrap().as_str()
        )
    }

    pub fn make_seq() -> H256 {
        let v: u32 = rand::random();
        let vhash = keccak(v.to_be_bytes());
        return vhash;
    }
    pub fn pack(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.append(&mut Vec::from((self.length as u32).to_be_bytes()));
        buffer.append(&mut Vec::from(self.packtype.to_be_bytes()));
        buffer.append(&mut Vec::from(self.seq.to_fixed_bytes()));
        buffer.append(&mut Vec::from(self.result.to_be_bytes()));
        buffer.append(&mut self.data.clone());
        buffer
    }

    pub fn unpack(data: &Vec<u8>) -> Result<ChannelPack, KissError> {
        if data.len() < 42 {
            return kisserr!(
                KissErrKind::EFormat,
                "channelpack data length too short:  {}",
                data.len()
            );
        }
        let mut pack = ChannelPack::default();
        //deserialize::<ChannelPack>(data.as_slice()).unwrap();
        pack.length = i32::from_be_bytes(data.as_slice()[..4].try_into().unwrap()) as usize;
        if data.len() < pack.length {
            //数据量不够
            return kisserr!(KissErrKind::EFormat, "buffer size less than pack.length");
        }
        let mut start = 4;
        pack.packtype = u16::from_be_bytes(data.as_slice()[start..(start + 2)].try_into().unwrap());
        start = start + 2;
        pack.seq = H256::from_slice(&data[start..start + 32]);
        start += 32;
        pack.result = u32::from_be_bytes(data.as_slice()[start..start + 4].try_into().unwrap());
        //start += 4;
        pack.data = Vec::from(&data[42..pack.length]);
        Ok(pack)
    }
}
pub fn make_channel_pack(packtype:CHANNEL_PACK_TYPE,data: &str) -> Option<ChannelPack> {
    make_channel_pack_by_rawdata(packtype,&Vec::from(data))
}


pub fn make_channel_pack_by_rawdata(packtype:CHANNEL_PACK_TYPE,data: &Vec<u8>) -> Option<ChannelPack> {
    let mut pack = ChannelPack::default();
    pack.data.append(&mut data.clone());
    pack.seq = ChannelPack::make_seq();
    pack.packtype = packtype as u16;
    pack.result = 0;
    pack.length = 42 + pack.data.len();
    //println!("****** request pack seq {:?}",pack.seq);
    //let bin = pack.pack();
    Option::from(pack)
}


///
///  1字节头部长度（1+topic长度）+ topic + data
pub fn pack_amop(topic:&Vec<u8>,data:&Vec<u8>)->Vec<u8>{
    let headerlen:u8 = 1+ (topic.len() as u8);
    let mut buffer:Vec<u8> = vec!();
    buffer.append(&mut Vec::from(headerlen.to_be_bytes()));
    buffer.append(&mut topic.clone());
    buffer.append(&mut data.clone());
    buffer
}
pub fn unpack_amop(buffer:&Vec<u8>)->(Vec<u8>,Vec<u8>)
{
    //let  start = 0;
    let headerlen =u8::from_be_bytes( buffer.as_slice()[0..1].try_into().unwrap() );
    let mut topic = vec!();
    if headerlen > 1 {
        topic = Vec::from(&buffer[1..headerlen as usize])
    }
    let mut data = vec!();
    data = Vec::from(&buffer[headerlen as usize..]);
    (topic,data)

}

pub fn test_channelpack() {
    let mut pack = ChannelPack::default();
    let data = "1234567890";
    pack.data = Vec::from(data);
    pack.seq = ChannelPack::make_seq();
    pack.packtype = 0x12;
    pack.result = 0;
    pack.length = 42 + pack.data.len();
    let bin = pack.pack();
    //let bin  = serialize(&pack).unwrap();
    println!("{:?}", &bin);
    let hexv = hex::encode(bin.as_slice());
    println!("{:?}", &hexv);
    println!("totallen = {}", hexv.len());

    let unpackres = ChannelPack::unpack(&bin).unwrap();
    if unpackres.seq != pack.seq {
        println!("ne");
    } else {
        println!("seq eq");
    }
    println!("unpack: {:?}", unpackres);
    println!("data content {}", String::from_utf8(pack.data).unwrap());
}
/*
https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/design/protocol_description.html#id5

0x12	JSONRPC 2.0格式	RPC接口消息包	SDK->节点
0x13	json格式心跳包{"heartbeat":"0"}	心跳包	0:SDK->节点，1:节点->SDK
0x14	SDK->节点的包体{"minimumSupport":version,"maximumSupport":version,"clientType":"client type"},节点->SDK的包体{"protocol":version,"nodeVersion":"fisco-bcos version"	握手包，json格式的协议版本协商	SDK<->节点，双向
0x30	AMOP消息包包体	AMOP请求包	SDK<->节点，双向
0x31	失败的AMOP消息的包体	AMOP失败响应包	节点->SDK或节点->节点
0x32	json数组，存储SDK监听的Topics	上报Topic信息	SDK->节点
0x35	AMOP消息包包体	AMOP多播消息	节点->节点
0x1000	json格式的交易上链通知	交易上链回调	节点->SDK
0x1001	json格式的区块上链通知{"groupID":"groupID","blockNumber":"blockNumber"}
*/
/*
https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/design/protocol_description.html#id6
错误码
code	message
0	成功
100	节点不可达
101	SDK不可达
102	超时
*/
