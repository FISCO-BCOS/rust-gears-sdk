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
/*
  FISCO BCOS/rust-SDK is a rust client for FISCO BCOS2.0 (https://github.com/FISCO-BCOS/)
  FISCO BCOS/rust-SDK is free software: you can redistribute it and/or modify it under the
  terms of the MIT License as published by the Free Software Foundation. This project is
  distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
  the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
  @author: kentzhang
  @date: 2021-07
*/
use reqwest::header::HeaderMap;

use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use crate::{kisserr, printlnex};

//---------------------------------------------------------------------------------
/// http的网络客户端
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HttpJsonRpcClient {
    pub target_url: String,
    pub timeout: u32,
    headers: HeaderMap,
}

impl HttpJsonRpcClient {
    pub fn new() -> HttpJsonRpcClient {
        // 组装header
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        HttpJsonRpcClient {
            target_url: "".to_string(),
            timeout: 10,
            headers: headers,
        }
    }
    pub fn set_target(&mut self, target_url: &str) {
        self.target_url = target_url.to_string();
    }

    ///同步的http请求，正确的话返回http的body，也就是一段json串
    pub fn request_sync(&self, outbuffer: &str) -> Result<String, KissError> {
        let client = reqwest::blocking::Client::new();

        /*todo 改成支持timeout*/
        printlnex!("request target url : {:?}", &self.target_url);
        let postResult = client
            .post(&self.target_url)
            .headers(self.headers.clone())
            .body(outbuffer.to_string())
            .send();
        //post 是否正确
        match postResult {
            Ok(response) => {
                //println!("{:?}", response);
                if !response.status().is_success() {
                    // let httpmsg = format!("http response status :{:?}", response.status().to_string());
                    return kisserr!(
                        KissErrKind::ENetwork,
                        "http response status :{:?}",
                        response.status().to_string()
                    );
                }
                //从resposne里获取文本（body）内容
                match response.text() {
                    Ok(text) => Ok(text),
                    Err(e) => {
                        kisserr!(KissErrKind::ENetwork, "get response text error {:?}", e)
                    }
                }
            } //http response result
            Err(e) => {
                kisserr!(KissErrKind::ENetwork, "post error {:?}", e)
            }
        }
    }
}
