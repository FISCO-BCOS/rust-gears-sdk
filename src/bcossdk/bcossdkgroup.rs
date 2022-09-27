/*---------------------------------
https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html
----------------------------------*/
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
use serde_json::{json, Value as JsonValue};
use crate::bcossdk::bcossdk::BcosSDK;
use crate::bcossdk::kisserror::{ KissError};
use chrono::Local; // timer

impl BcosSDK {
    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#generategroup
    /// '{"jsonrpc":"2.0","method":"generateGroup","params":[2, {"timestamp":"1585214879000","sealers":["70f18c055d366615e86df99f91b6d3f16f07d66293b203b73498442c0366d2c8ff7a21bb56923d9d81b1c2916251888e47adf66c350738c898defac50aead5ab","dde37f534885f08db914566efeb03183d59363a4be972bbcdde25c37f0b350e1980a7de4fdc4aaf956b931aab00b739a8af475ed2461b8591d8c734b27285f57","d41672b29b3b1bfe6cad563d0f0b2a2735865b27918307b85085f892043a63f681ac8799243e920f7bb144b111d854d0592ba5f28aa7a4e0f9f533f9fdf76ead","7ba2717f81f38e7371ccdbe173751f051b86819f709e940957664dbde028698fd31ba3042f7dd9accd73741ba42afc35a8ef67fe7abbdeb76344169773aa0eca"],"enable_free_storage":true}],"id":1}' http://127.0.0.1:8545 | jq
    //
    // // Result
    // {
    //   "id": 1,
    //   "jsonrpc": "2.0",
    //   "result": {
    //     "code": "0x0",
    //     "message": "Group 2 generated successfully"
    //   }
    // }
    pub fn generateGroup(&mut self,groupid:u32,sealers:&Vec<String>,enable_free_storage:bool)->Result<JsonValue, KissError> {
        let cmd = "generateGroup";
        let timestamp =  format!("{}", Local::now().timestamp_millis());
        let paramobj = json!([
            groupid,
            {
                "timestamp": timestamp,
                "sealers": sealers,
                "enable_free_storage": enable_free_storage
            }
        ]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(v["result"].clone())
    }

    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#startgroup
    pub fn startGroup(&mut self,groupid:u32)->Result<JsonValue, KissError> {
        let cmd = "startGroup";
        let timestamp =  Local::now().timestamp_millis();
        let paramobj = json!([groupid]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(v["result"].clone())
    }


    ///https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html#startgroup
    pub fn stopGroup(&mut self,groupid:u32)->Result<JsonValue, KissError> {
        let cmd = "stopGroup";
        let timestamp =  Local::now().timestamp_millis();
        let paramobj = json!([groupid]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(v["result"].clone())
    }



    pub fn removeGroup(&mut self,groupid:u32)->Result<JsonValue, KissError> {
        let cmd = "removeGroup";
        let paramobj = json!([groupid]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(v["result"].clone())
    }


    pub fn recoverGroup(&mut self,groupid:u32)->Result<JsonValue, KissError> {
        let cmd = "recoverGroup";
        let paramobj = json!([groupid]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(v["result"].clone())
    }

    pub fn queryGroupStatus(&mut self,groupid:u32)->Result<JsonValue, KissError> {
        let cmd = "queryGroupStatus";
        let paramobj = json!([groupid]);
        let v: JsonValue = self.netclient.rpc_request_sync(cmd, &paramobj)?;
        Ok(v["result"].clone())
    }

}
