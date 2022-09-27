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
use serde::{Deserialize, Serialize};
use serde_json::{json, Result as JsonResult, Value as JsonValue};

use crate::bcossdk::bcos_channel_client::{BcosChannelClient, IBcosChannel};
use crate::bcossdk::bcosclientconfig::{BcosClientProtocol, ClientConfig};
use crate::bcossdk::bcoshttpclient::HttpJsonRpcClient;
use crate::bcossdk::kisserror::{KissErrKind, KissError};

///对应json rpc的request json格式
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RpcRequestData {
    pub jsonrpc: String,
    pub method: String,
    pub params: JsonValue,
    pub id: u8,
}

impl RpcRequestData {
    fn default() -> Self {
        RpcRequestData {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "".to_string(),
            params: json! {[1]},
        }
    }
    ///采用serde_json encode为json格式
    pub fn encode(&self) -> Result<String, KissError> {
        let res = serde_json::to_string(self);
        match res {
            Ok(jsonstr) => Ok(jsonstr),
            Err(e) => {
                kisserr!(
                    KissErrKind::EFormat,
                    "rpc request encode to json error {:?}, {:?}",
                    self,
                    e
                )
            }
        }
    }
    ///采用serde_json 从json格式
    pub fn decode(jsonstr: String) -> JsonResult<RpcRequestData> {
        let decode_res = serde_json::from_str(jsonstr.as_str())?;
        Ok(decode_res)
    }
}

//-----------------------------------------------------------------------------------
///统一对外暴露这个实现,封装向网络提交请求的部分
#[derive()]
pub struct BcosRPC {
    //http rpc 的实现
    pub config: ClientConfig,
    pub jsonrpc_client: HttpJsonRpcClient,
    pub channel_client: BcosChannelClient,
}
//unsafe impl Send for BcosRPC{}
//unsafe impl Sync for BcosRPC{}
impl BcosRPC {
    pub fn new(config: &ClientConfig) -> Result<BcosRPC, KissError> {
        //默认建一个json rpc的对象
         //           printlnex!("new channel_client");
        let mut jsonrpc_client = HttpJsonRpcClient::new();
        jsonrpc_client.target_url = config.rpc.url.clone();
        jsonrpc_client.timeout = config.rpc.timeout;
        let channel_client:BcosChannelClient;
        if config.bcos2.protocol == BcosClientProtocol::CHANNEL {
            channel_client = BcosChannelClient::new(&config.channel)?;
        }else{

            channel_client = BcosChannelClient::default(&config.channel);
            printlnex!("done channel_client");
        }
        Ok(BcosRPC {
            config: config.clone(),
            jsonrpc_client: jsonrpc_client,
            channel_client: channel_client,
        })
    }

    pub fn finish(&mut self) {
        self.channel_client.finish();
    }
    pub fn switch_rpc_request_sync(&mut self, outbuffer: &String) -> Result<String, KissError> {
        //let mut response_text =String::default();
        match self.config.bcos2.protocol {
            BcosClientProtocol::RPC => self.jsonrpc_client.request_sync(&outbuffer),
            BcosClientProtocol::CHANNEL => self.channel_client.request_sync(&outbuffer),
            _=>{return kisserr!(KissErrKind::EArgument,"unhandled protocal {:?}",self.config.bcos2.protocol)}
        }
        //Ok(response_text);
    }

    ///同步调用的客户端请求，输入cmd，如 getBlockNumber，value:参数，参考bcos rpc接口文档，参数中应包含groupid
    /// todo：异步请求待实现
    pub fn rpc_request_sync(
        &mut self,
        cmd: &str,
        params_value: &JsonValue,
    ) -> Result<JsonValue, KissError> {
        log::debug!("rpc_request_sync cmd {:?},{:?}",cmd,params_value);
        let req = RpcRequestData {
            method: cmd.to_string(),
            params: params_value.clone(),
            ..RpcRequestData::default()
        };
        let outbuffer = req.encode()?;
        printlnex!("request: {:?}", outbuffer);
        log::info!("request: {:?}", outbuffer);
        let responsebuffer = self.switch_rpc_request_sync(&outbuffer)?;
        log::info!("response:  {:?}", &responsebuffer);
        let jsonres: JsonResult<JsonValue> = serde_json::from_str(responsebuffer.as_str());
        match jsonres {
            Ok(jsonval) => {
                printlnex!("request response: {:?}", jsonval);
                Ok(jsonval)
            }
            Err(e) => {
                log::error!("parse json rpc response json error {},{:?}",
                    responsebuffer,
                    e);
                return kisserr!(
                    KissErrKind::EFormat,
                    "parse json rpc response json error {},{:?}",
                    responsebuffer,
                    e
                );
            }
        }
    }
}

//----------------------------------------------------------------------
pub fn test_json_rpc() {
    let groupid = 1;
    let config = ClientConfig::load("conf/client_config.toml").unwrap();
    let mut client = BcosRPC::new(&config).unwrap();
    let params = &json!([groupid]);
    let response = client.rpc_request_sync("getBlockNumber", params);
    println!("{:?}", response);
}
