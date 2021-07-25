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

use serde_derive::Deserialize;
use toml;

use crate::bcossdk::fileutils;
use crate::bcossdk::kisserror::{KissErrKind, KissError};

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum BcosCryptoKind {
    GM,
    ECDSA,
}
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum BcosClientProtocol {
    RPC,
    CHANNEL,
}

impl BcosCryptoKind {
    pub fn default() -> Self {
        BcosCryptoKind::ECDSA
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChainConfig {
    pub chainid: u32,
    pub groupid: u32,
    pub crypto: BcosCryptoKind,
    pub accountpem: String,
    pub protocol: BcosClientProtocol,
}
impl ChainConfig {
    pub fn default() -> Self {
        ChainConfig {
            chainid: 1,
            groupid: 1,
            crypto: BcosCryptoKind::ECDSA,
            accountpem: "".to_string(),
            protocol: BcosClientProtocol::RPC,
        }
    }
}
//rpc连接方式的配置
#[derive(Deserialize, Debug, Default, Clone)]
pub struct RpcConfig {
    pub url: String,
    pub timeout: u32, //in sec
}
impl RpcConfig {
    pub fn default() -> Self {
        RpcConfig {
            url: "".to_string(),
            timeout: 10,
        }
    }
}

///channel连接方式的配置
#[derive(Deserialize, Debug, Clone)]
pub struct ChannelConfig {
    pub ip: String,
    pub port: u32,
    pub timeout: u32,
    pub tlskind: BcosCryptoKind, //tls握手加密方式，ecdsa或国密
    pub nativelib_echo_mode: u32,
    pub cacert: String,
    pub sdkcert: String,
    pub sdkkey: String,

    pub gmcacert: String,
    pub gmsdkcert: String,
    pub gmsdkkey: String,
    pub gmensdkcert: String,
    pub gmensdkkey: String,
}
impl ChannelConfig {
    pub fn default() -> Self {
        ChannelConfig {
            ip: "".to_string(),
            port: 0,
            tlskind: BcosCryptoKind::ECDSA,
            nativelib_echo_mode: 0,
            cacert: "sdk/ca.crt".to_string(),
            sdkcert: "sdk/sdk.crt".to_string(),
            sdkkey: "sdk/sdk.key".to_string(),
            gmcacert: "sdk/gmca.crt".to_string(),
            gmsdkcert: "sdk/gmsdk.crt".to_string(),
            gmsdkkey: "sdk/gmsdk.key".to_string(),
            gmensdkcert: "sdk/gmensdk.crt".to_string(),
            gmensdkkey: "sdk/gmensdk.key".to_string(),
            timeout: 10,
        }
    }
}

///合约相关配置，主要是目录和历史保存路径
#[derive(Deserialize, Debug, Clone)]
pub struct ContractConfig {
    pub contractpath: String,
    pub solc :String, //solc编译器
    pub solcgm :String, //solc国密版本编译器
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientConfig {
    pub chain: ChainConfig,
    pub contract: ContractConfig,
    pub rpc: RpcConfig,
    pub channel: ChannelConfig,
    pub configfile: Option<String>,
}

impl ClientConfig {
    pub fn load(config_file: &str) -> Result<ClientConfig, KissError> {
        let loadres = fileutils::readstring(config_file);
        match loadres {
            Ok(text) => {
                //println!("{:?}",text);
                let configresult: Result<ClientConfig, toml::de::Error> = toml::from_str(&text);
                match configresult {
                    Ok(config) => {
                        let mut c = config.clone();
                        c.configfile = Option::from(config_file.to_string());
                        Ok(c)
                    }
                    Err(e) => {
                        kisserr!(
                            KissErrKind::EFormat,
                            "parse toml file error {},{:?}",
                            config_file,
                            e
                        )
                    }
                }
            }
            Err(e) => {
                return kisserr!(
                    KissErrKind::Error,
                    "load config error {},{:?}",
                    config_file,
                    e
                )
            }
        }
    }
}

//------------------------------------------------------------------------
pub fn test_config() {
    let res = ClientConfig::load("conf/client_config.toml");
    println!("{:?}", res);
}
