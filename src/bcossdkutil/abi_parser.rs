/*
  FISCO BCOS/rust-SDK is a rust client for FISCO BCOS2.0 (https://github.com/FISCO-BCOS/)
  FISCO BCOS/rust-SDK is free software: you can redistribute it and/or modify it under the
  terms of the MIT License as published by the Free Software Foundation. This project is
  distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
  the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
  @author: kentzhang
  @date: 2021-07
*/

/*
2021.08，建这个文件是为了解决ethabi-12.0.0项目本身解析时的一些问题，比如
1）解析参数字符串时，简单的按“,”分割，当输入字符串里也包含,尤其是类似数组、tuple等,解析会出现问题
2)参数里未实现tuple数组的完整解析，event也不支持tuple数组输出，对tuple数组，仅解析成tuple[],而不是类似(uinit256,string)[]这种
于是干脆从abi文件解析实现，复用ethabi里的Event,ParamType等基本元素，解决以上问题
*/
#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_imports,
    unused_variables,
    unused_assignments
)]

use hex::ToHex;
use hex_literal::hex;
use keccak_hash::keccak;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs::File;

use crate::bcossdkutil::commonhash::{CommonHash, HashType};
use crate::bcossdkutil::event_utils::EventABIUtils;
use crate::bcossdkutil::fileutils::read_all;
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use ethabi::{Event, EventParam, ParamType};
use pem::parse;

#[derive(Clone, Debug)]
pub struct ABIParser {
    pub abitext: String,
    pub abiobj: JsonValue,
    pub events: Vec<Event>,
}

impl ABIParser {
    ///从文件加载abi文本
    /// 解析成json对象
    pub fn load(filename: &str) -> Result<ABIParser, KissError> {
        let content = read_all(filename)?;
        let abitext = String::from_utf8(content).unwrap();
        let abiobj: JsonValue = serde_json::from_str(abitext.as_str()).unwrap();
        let mut parser = ABIParser {
            abitext: abitext,
            abiobj: abiobj,
            events: vec![],
        };
        parser.parse();
        Ok(parser)
    }

    ///解析结构体参数类型，type是 tuple,每个参数都是component
    pub fn parse_tuple_params(input: &JsonValue) -> ParamType {
        let mut tupleparams: Vec<Box<ParamType>> = vec![];
        for c in input["components"].as_array().unwrap() {
            //let typename = c["type"].as_str().unwrap();
            let paramtype = ABIParser::parse_param_type(c, "").unwrap();
            tupleparams.push(Box::new(paramtype));
        }
        let tupleparam = ParamType::Tuple(tupleparams);
        tupleparam
    }

    ///根据参数的type，构建ParamType对象，spectypename的意思是由调用者指定type名字
    /// 例如 type是tuple[],实际上按tuple解析成类似 (string,uint256)[]这样的形式，就需要指定spectypename= 'tuple'
    pub fn parse_param_type(input: &JsonValue, spectypename: &str) -> Result<ParamType, KissError> {
        let mut typename: String = spectypename.trim().to_string();
        if typename.len() == 0 {
            let typenameOpt = input["type"].as_str();
            typename = match typenameOpt {
                Some(s) => s.to_string(),
                None => {
                    return kisserr!(KissErrKind::EArgument, "miss param type name");
                }
            };
        }
        typename = typename.trim().to_string(); //去掉可能的空格
        let result = match typename.as_str() {
            "address" => ParamType::Address,
            "bytes" => ParamType::Bytes,
            "bool" => ParamType::Bool,
            "string" => ParamType::String,
            "int" => ParamType::Int(256),
            "tuple" => ABIParser::parse_tuple_params(input),
            "uint" => ParamType::Uint(256),
            s if s.starts_with("int") => {
                let len = usize::from_str_radix(&s[3..], 10).unwrap();
                ParamType::Int(len)
            }
            s if s.starts_with("uint") => {
                let len = usize::from_str_radix(&s[4..], 10).unwrap();
                ParamType::Uint(len)
            }
            s if s.starts_with("bytes") => {
                let len = usize::from_str_radix(&s[5..], 10).unwrap();
                ParamType::FixedBytes(len)
            }
            s if s.ends_with(']') => {
                //数组类型，类似string[],string[3],tuple[]等
                let items: Vec<&str> = s.split("[").collect();
                let paramtype = ABIParser::parse_param_type(input, &items[0]).unwrap();
                ParamType::Array(Box::new(paramtype))
            }
            _ => {
                return kisserr!(KissErrKind::EArgument, "unknow typename {}", typename);
            }
        };
        Ok(result)
    }

    ///解析所有的event定义
    pub fn parse_event_inputs(&self, inputlist: &JsonValue) -> Vec<EventParam> {
        let mut params: Vec<EventParam> = vec![];
        for input in inputlist.as_array().unwrap() {
            let orgtype = input["type"].as_str().unwrap();
            let paramtype = ABIParser::parse_param_type(&input, "").unwrap();
            let param = EventParam {
                name: input["name"].as_str().unwrap().to_string(),
                indexed: input["indexed"].as_bool().unwrap(),
                kind: paramtype,
            };
            params.push(param);
        }
        params
    }
    ///解析单个event对象
    pub fn parse_event(&self, item: &JsonValue) -> Event {
        let name = item["name"].as_str().unwrap();
        let anonymous = item["anonymous"].as_bool().unwrap();
        let event = Event {
            name: name.to_string(),
            inputs: self.parse_event_inputs(&item["inputs"]),
            anonymous: anonymous,
        };
        let event_abi_utils = EventABIUtils::new(&HashType::WEDPR_KECCAK);
        let sig = event_abi_utils.event_signature(&event);
        //println!("abiparser->event sig:{} {:?}", hex::encode(sig), event);
        event
    }

    ///解析的总入口
    ///todo function等定义。
    pub fn parse(&mut self) {
        self.events.clear();
        for item in self.abiobj.as_array().unwrap() {
            match item["type"].as_str().unwrap() {
                "event" => {
                    let event = self.parse_event(&item);
                    self.events.push(event);
                }

                __ => {}
            }
        }
    }
}
/*
        println!("abistr {}",abistr);
        let abijson:JsonValue = serde_json::from_str(abistr.as_str()).unwrap();
        let items =  abijson.as_array().unwrap();
        for item in items{
            if item["type"] == "event" && item["name"]=="onaddusers"{
                //println!("item {:?}", item["inputs"]);
                for param in item["inputs"].as_array().unwrap(){
                    let paramtype = &param["type"].as_str().unwrap();
                    println!("param type [{}]",paramtype);
                    if paramtype.starts_with("tuple") {
                        println!("param type is tuple{},{:?}",paramtype,param);
                        let mut types:Vec<String> = vec!();
                        for component in param["components"].as_array().unwrap(){
                            println!("component type {:?}",component["type"]);
                            types.push(component["type"].as_str().unwrap().to_string());
                        }
                        let tupletype = format!("({})[]",types.join(","));
                        let data: Vec<u8> = From::from(format!("{}(uint256,{})", "onaddusers", tupletype).as_str());
                        println!("signature: {}",String::from_utf8(data.clone()).unwrap());
                        let hashbytes = CommonHash::hash(&data,&HashType::WEDPR_KECCAK);
                        println!("hashbytes in hex: {}",hex::encode(hashbytes));
                    }
                }

            }
        }

*/
