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
//#[macro_use]
use lazy_static::lazy_static;
/*封装hash方法，包括wedpr的hash实现，以支持ECDSA和国密的hash*/
use crate::bcossdkutil::bcosclientconfig::BcosCryptoKind;
use keccak_hash::{keccak, H256};
use wedpr_l_crypto_hash_keccak256::WedprKeccak256;
use wedpr_l_crypto_hash_sm3::WedprSm3;
use wedpr_l_utils::traits::Hash;

#[derive(Debug, Clone, PartialEq)]
pub enum HashType {
    WEDPR_KECCAK,
    WEDRP_SM3,
    KECCAK,
    Unknow, //默认值，当未指定时，调用会抛异常，相对用错了hash方法，更容易定位问题
}
lazy_static! {
    static ref WEDPR_SM3: WedprSm3 = WedprSm3::default();
    static ref WEDPR_KECCAK256: WedprKeccak256 = WedprKeccak256::default();
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommonHash {
    hashtype: HashType,
}

impl CommonHash {
    ///因为这里的hash实现设计目的是可以切换多种hash算法，包括国密，非国密，keccak以及其他等，要求调用方传入type
    /// 之所以不采用全局的设置，是考虑到兼容在同一个进程里不同的调用者会使用不同的算法
    pub fn hash(data: &Vec<u8>, hashtype: &HashType) -> Vec<u8> {
        printlnex!("Using HashType {:?}", hashtype);
        match hashtype {
            HashType::WEDPR_KECCAK => {
                let msg_hash = WEDPR_KECCAK256.hash(data.as_slice());
                msg_hash
            }
            HashType::WEDRP_SM3 => {
                let msg_hash = WEDPR_SM3.hash(data.as_slice());
                //printlnex!("msg_hash: {:?}",msg_hash);
                msg_hash
            }
            HashType::KECCAK => {
                let keccakhash = keccak(&data);
                Vec::from(keccakhash.as_bytes())
            }
            HashType::Unknow => {
                //！！！返回一个空值,调用者如校验hash值必然会出错，总比用错了算法查不出所以然好
                vec![]
            }
        }
    }

    pub fn hash_to_h256(data: &Vec<u8>, hashtype: &HashType) -> H256 {
        let hash = CommonHash::hash(data, hashtype);
        let h256 = H256::from_slice(&hash.as_slice());
        h256
    }

    pub fn crypto_to_hashtype(crypto: &BcosCryptoKind) -> HashType {
        match crypto {
            BcosCryptoKind::ECDSA => {
                //一定要先设置hash算法，这是基础中的基础
                return HashType::WEDPR_KECCAK;
            }
            BcosCryptoKind::GM => {
                //一定要先设置hash算法，这是基础中的基础
                return HashType::WEDRP_SM3;
            }
        }
    }
}
