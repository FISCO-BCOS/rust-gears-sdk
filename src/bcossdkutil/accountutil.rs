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
use pem::Pem;
use wedpr_l_crypto_signature_secp256k1::WedprSecp256k1Recover;
use wedpr_l_crypto_signature_sm2::WedprSm2p256v1;
use wedpr_l_libsm::sm2::signature::SigCtx;
use wedpr_l_utils::traits::Signature;

use crate::bcossdkutil::bcosclientconfig::BcosCryptoKind;
use crate::bcossdkutil::commonhash::{CommonHash, HashType};
use crate::bcossdkutil::fileutils;
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use pkcs8::PrivateKeyInfo;
use std::convert::TryFrom;

///常用账户熟悉和方法的封装
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BcosAccount {
    pub privkey: Vec<u8>,
    pub pubkey: Vec<u8>,
    pub address: Vec<u8>,
}
//unsafe impl Send for BcosAccount{}
//unsafe impl Sync for BcosAccount{}
impl BcosAccount {
    ///将Account转成hex字符格式
    pub fn to_hexdetail(&self) -> String {
        let str = format!(
            "address: {}\nprivkey: {}\npubkey: {}",
            hex::encode(&self.address),
            hex::encode(&self.privkey),
            hex::encode(&self.pubkey)
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
        Ok(pem) => {
            let privkey = try_from_fisco_pem_format(pem.contents.clone());
            match privkey {
                Some(v) => Ok(v),
                None => Ok(pem.contents),
            }
        }
        Err(e) => {
            kisserr!(KissErrKind::EFormat, "load pem {:?} error {:?}", pemfile, e)
        }
    }
}

#[derive(asn1::Asn1Read, asn1::Asn1Write, Debug)]
pub struct FiscoPrivateKey<'a> {
    Version: u64,
    PrivateKey: &'a [u8],
    #[explicit(0)]
    NamedCurveOID: Option<asn1::ObjectIdentifier>,
    #[explicit(1)]
    PublicKey: Option<asn1::BitString<'a>>,
}

/// 尝试Fisco console生成的PKCS8格式解析
pub fn try_from_fisco_pem_format(contents: Vec<u8>) -> Option<Vec<u8>> {
    if let Ok(private_key_info) = PrivateKeyInfo::try_from(contents.as_ref()) {
        if let Ok(fisco_private_key) =
            asn1::parse_single::<FiscoPrivateKey>(private_key_info.private_key)
        {
            return Some(fisco_private_key.PrivateKey.to_vec());
        }
    }
    None
}

fn address_from_pubkey(pubkey: &Vec<u8>, hashtype: &HashType) -> Vec<u8> {
    let mut actpubkey = pubkey.clone();
    if pubkey.len() == 65 {
        actpubkey = actpubkey[1..].to_vec(); //去掉头部的压缩标记
    }
    let hash = CommonHash::hash(&actpubkey, hashtype);
    let addressbytes = hash[12..].to_vec();
    addressbytes
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

lazy_static! {
    static ref WEDPRSM2: WedprSecp256k1Recover = WedprSecp256k1Recover::default();
}

impl IBcosAccountUtil for EcdsaAccountUtil {
    ///创建一个基于随机数的账户
    fn create_random(&self) -> BcosAccount {
        let (pubkey, secret_key) = WEDPRSM2.generate_keypair();
        let address = address_from_pubkey(&pubkey, &HashType::KECCAK);
        BcosAccount {
            privkey: secret_key,
            pubkey: pubkey,
            address: address,
        }
    }
    ///从私钥字节转为账户
    fn from_privkey_bytes(&self, privkey: &Vec<u8>) -> Result<BcosAccount, KissError> {
        let keyresult = WEDPRSM2.derive_public_key(privkey);
        match keyresult {
            Ok(pubkey) => {
                let account = BcosAccount {
                    privkey: privkey.clone(),
                    pubkey: pubkey.clone(),
                    address: address_from_pubkey(&pubkey, &HashType::KECCAK),
                };
                Ok(account)
            }
            Err(e) => {
                kisserr!(KissErrKind::Error, "from privakey to pub key error {:?}", e)
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
        //printlnex!("pubkey is  {:?}", pubkey);
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
