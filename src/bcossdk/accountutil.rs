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

use lazy_static::lazy_static;
#[allow(unused_imports)]
use parity_crypto::publickey::{Generator, KeyPair, Random};
use pem::Pem;
use rustc_hex::ToHex;
use wedpr_l_crypto_signature_sm2::WedprSm2p256v1;
use wedpr_l_libsm::sm2::signature::SigCtx;
use wedpr_l_utils::traits::Signature;

use crate::bcossdk::bcosclientconfig::BcosCryptoKind;
use crate::bcossdk::commonhash::{CommonHash, HashType};
use crate::bcossdk::fileutils;
use crate::bcossdk::kisserror::{KissErrKind, KissError};

///常用账户熟悉和方法的封装
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BcosAccount {
    pub privkey: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub address: Vec<u8>,
}

impl BcosAccount {
    ///将Account转成hex字符格式
    pub fn to_hexdetail(&self) -> String {
        let str = format!(
            "address: {}\nprivkey: {}\npubkey: {}",
            self.address.to_hex(),
            self.privkey.to_hex(),
            self.pubkey.to_hex()
        );
        str
    }
}

//helper 方法

///将私钥存入pem文件,无密码
pub fn save_key_to_pem(key: &Vec<u8>, pemfile: &str) -> Result<(), KissError> {
    let content = Pem {
        tag: String::from("PRIVATE KEY"),
        contents: key.clone(),
    };
    fileutils::write_all(pemfile, pem::encode(&content).into())
}

///从pem文件加载私钥
pub fn load_key_from_pem(pemfile: &str) -> Result<Vec<u8>, KissError> {
    let key = fileutils::read_all(pemfile)?;
    let pemres = pem::parse(key);
    match pemres {
        Ok(pem) => Ok(pem.contents),
        Err(e) => {
            kisserr!(KissErrKind::EFormat, "load pem {:?} error {:?}", pemfile, e)
        }
    }
}

pub trait IBcosAccountUtil {
    ///创建随机账户
    fn create_random(&self) -> BcosAccount;
    ///从私钥字节转换为账户
    fn from_privkey_bytes(&self, privkey: &Vec<u8>) -> Result<BcosAccount, KissError>;
    ///从pem文件里的私钥转换为账户
    fn from_pem(&self, pemfile: &str) -> Result<BcosAccount, KissError>;
}

///ecdsa 账户，实现创建和从既有私钥（字节）生成
///在语义上和国密的区分，
#[derive(Default, Debug)]
pub struct EcdsaAccountUtil {}

impl IBcosAccountUtil for EcdsaAccountUtil {
    ///创建一个基于随机数的账户
    fn create_random(&self) -> BcosAccount {
        let key = Random.generate();
        BcosAccount {
            privkey: key.secret().as_bytes().into(),
            pubkey: key.public().as_bytes().into(),
            address: key.address().as_bytes().into(),
        }
    }
    ///从私钥字节转为账户
    fn from_privkey_bytes(&self, privkey: &Vec<u8>) -> Result<BcosAccount, KissError> {
        let keyresult = KeyPair::from_secret_slice(privkey.as_slice());
        match keyresult {
            Ok(key) => {
                let account = BcosAccount {
                    privkey: key.secret().as_bytes().into(),
                    pubkey: key.public().as_bytes().into(),
                    address: key.address().as_bytes().into(),
                };
                Ok(account)
            }
            Err(e) => {
                kisserr!(KissErrKind::Error, "turn data to key error {:?}", e)
            }
        }
    }
    ///从pem文件加载私钥,并转为账户格式,无密码
    fn from_pem(&self, pemfile: &str) -> Result<BcosAccount, KissError> {
        let key = load_key_from_pem(pemfile)?;
        self.from_privkey_bytes(&key)
    }
}

//--------------------国密实现------------------------------------------

fn address_from_pubkey(pubkey: &Vec<u8>, hashtype: &HashType) -> Vec<u8> {
    let hash = CommonHash::hash(&pubkey, hashtype);
    let addressbytes = hash[12..].to_vec();
    addressbytes
}

lazy_static! {
    // Shared sm2 instance initialized for all functions.
    static ref SM2_CTX: SigCtx = SigCtx::new();
    static ref WEDPR_SM2 : WedprSm2p256v1= WedprSm2p256v1::default();
}

///国密账户的功能接口
#[derive(Default, Debug)]
pub struct GMAccountUtil {}

impl IBcosAccountUtil for GMAccountUtil {
    ///创建一个基于随机数的账户（国密）
    fn create_random(&self) -> BcosAccount {
        let (pubkey, privkey) = WEDPR_SM2.generate_keypair();
        BcosAccount {
            privkey: privkey,
            pubkey: pubkey.clone(),
            address: address_from_pubkey(&pubkey, &HashType::WEDRP_SM3),
        }
    }
    ///从私钥字节转为账户
    fn from_privkey_bytes(&self, privkey: &Vec<u8>) -> Result<BcosAccount, KissError> {
        let secret_key = match SM2_CTX.load_seckey(&privkey.as_ref()) {
            Ok(v) => v,
            Err(_) => {
                return kisserr!(KissErrKind::EFormat, "SM2_CTX.load_seckey");
            }
        };
        let derived_public_key = SM2_CTX.pk_from_sk(&secret_key);
        let mut pubkey = SM2_CTX.serialize_pubkey(&derived_public_key, false);
        printlnex!("pubkey is  {:?}", pubkey);
        if pubkey.len() == 65 {
            pubkey = pubkey[1..].to_vec();
        }
        let address = address_from_pubkey(&pubkey, &HashType::WEDRP_SM3);
        let account = BcosAccount {
            privkey: privkey.clone(),
            pubkey: pubkey,
            address: address,
        };
        Ok(account)
    }

    ///从pem文件加载私钥,并转为账户格式,无密码
    fn from_pem(&self, pemfile: &str) -> Result<BcosAccount, KissError> {
        let key = load_key_from_pem(pemfile)?;
        self.from_privkey_bytes(&key)
    }
}

///helper function,create new account according crypto type
pub fn create_account(cryptokind: &BcosCryptoKind) -> BcosAccount {
    match cryptokind {
        BcosCryptoKind::ECDSA => EcdsaAccountUtil::default().create_random(),
        BcosCryptoKind::GM => GMAccountUtil::default().create_random(),
    }
}

pub fn account_from_pem(
    pemfile: &str,
    cryptokind: &BcosCryptoKind,
) -> Result<BcosAccount, KissError> {
    match cryptokind {
        BcosCryptoKind::ECDSA => {
            let accountutil = EcdsaAccountUtil::default();
            accountutil.from_pem(pemfile)
        }
        BcosCryptoKind::GM => {
            let accountutil = GMAccountUtil::default();
            accountutil.from_pem(pemfile)
        }
    }
}

pub fn account_from_privkey(
    keybytes: &Vec<u8>,
    cryptokind: BcosCryptoKind,
) -> Result<BcosAccount, KissError> {
    match cryptokind {
        BcosCryptoKind::ECDSA => {
            let accountutil = EcdsaAccountUtil::default();
            accountutil.from_privkey_bytes(keybytes)
        }
        BcosCryptoKind::GM => {
            let accountutil = GMAccountUtil::default();
            accountutil.from_privkey_bytes(keybytes)
        }
    }
}

//测试代码开始--------------------------------------------
pub fn test_account() {
    let fixkey = "82dcd33c98a23d5d06f9331554e14ab4044a1d71b169b7a38b61c214f0690f80";
    //let account = EcdsaAccount::creat_random();
    let accountresult =
        EcdsaAccountUtil::default().from_privkey_bytes(&hex::decode(String::from(fixkey)).unwrap());
    let account = accountresult.unwrap();
    println!("account : {:?}", account);
    let pemfile = "sdk/test.pem";
    let res = save_key_to_pem(&account.privkey, pemfile);
    let loadres = load_key_from_pem(pemfile);
    let accountload = EcdsaAccountUtil::default().from_privkey_bytes(&loadres.unwrap());
    println!("load result {:?}", accountload);
    println!("account in hex : {:?}", account.to_hexdetail());
}
