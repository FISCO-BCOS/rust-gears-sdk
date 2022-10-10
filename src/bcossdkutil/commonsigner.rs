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

use std::convert::From;

//#[macro_use]
use lazy_static::lazy_static;
use wedpr_l_crypto_signature_secp256k1::WedprSecp256k1Recover;
use wedpr_l_crypto_signature_sm2::WedprSm2p256v1;
use wedpr_l_libsm::sm2::signature::Signature as WEDPRSM2Signature;
use wedpr_l_utils::traits::Signature;

use crate::bcossdkutil::accountutil::{BcosAccount, EcdsaAccountUtil, IBcosAccountUtil};
use crate::bcossdkutil::accountutil::GMAccountUtil;
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};

///secp256原始方式的签名串, * Ecdsa的签名结构和国密略有不同,国密的v直接就是公钥
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CommonSignature {
    // pub data :Vec<u8>,
    pub v: Vec<u8>,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
}

///无论是ecdsa,还是国密，前64字节都是r,s,64字节后，ecdsa是v，通常一个字节有效，国密是64个字节的公钥（不含压缩标记）
impl CommonSignature {
    pub fn detail(&self) -> String {
        format!(
            "r:{:?},s:{:?},v:{:?}",
            hex::encode(&self.r),
            hex::encode(&self.s),
            hex::encode(&self.v)
        )
    }
    pub fn from_vec(data: &Vec<u8>) -> Self {
        let r_ = data[0..32].to_vec();
        let s_ = data[32..64].to_vec();
        let v_ = data[64..].to_vec();
        CommonSignature {
            r: r_,
            s: s_,
            v: v_,
        }
    }

    pub fn from_rsv(r: &[u8], s: &[u8], v: &[u8]) -> Self {
        CommonSignature {
            r: r.to_vec(),
            s: s.to_vec(),
            v: v.to_vec(),
        }
    }
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];
        buffer.append(&mut self.r.clone());
        buffer.append(&mut self.s.clone());
        buffer.append(&mut self.v.clone());
        buffer
    }
}

///一些secp256的特有方法
pub struct Secp256Signature {}

//ecdsa对签名串的一些处理方法，主要是处理chainid,实际上bcos也不是很关心chainid了，这里的代码仅供参考，备不时所需
impl Secp256Signature {
    ///将v从64位转成8位
    ///bcos不需要支持魔术数字27以及chainid,这里的逻辑不重要了
    pub fn make_stand_v(v: u64) -> u64 {
        match v {
            v if v >= 27 && v <= 28 => v - 27,
            v if v >= 35 => (v - 1) % 2,
            _ => 4,
        }
    }
    ///将v从8位转成64位
    ///bcos不需要支持魔术数字27以及chainid,这里的逻辑不太重要了
    pub fn adjust_v_value(v: u64) -> u64 {
        if v != 4 {
            v as u64 + 27
        } else {
            v as u64
        }
    }

    pub fn adjust_v(v: u8) -> u64 {
        if v <= 1 || v == 4 {
            v as u64 + 27
        } else {
            v as u64
        }
    }
    //如果v=0/1，强行加个27,返回一个副本，不修改本地数据
    pub fn to_electrum(data: &Vec<u8>) -> Vec<u8> {
        let mut vout = data.clone();
        if vout[64] <= 1 {
            vout[64] += 27;
        }
        vout
    }
}

///-----------------------------------------------------
/// 屏蔽掉国密非国密，算法细节等，获得一个签名串
/// 调用者再根据签名串的具体算法，用特定对象（如Secp256signature之类的解析它
pub trait ICommonSigner {
    fn sign(&self, data: Vec<u8>) -> Result<CommonSignature, KissError>;
}
/*
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CommonSignerSecp256 {
    private_key: Vec<u8>,
}

impl ICommonSigner for CommonSignerSecp256 {
    fn sign(&self, data: Vec<u8>) -> Result<CommonSignature, KissError> {
        printlnex!("curr signer is {:?}", self);
        let secret = self.keypair().unwrap().secret().clone();
        let sig = publickey::sign(&secret, &H256::from_slice(data.as_slice()));
        match sig {
            Ok(signature) => {
                //采用secp256签名后，直接调整v的值，如果是0,1则+27 ，4不变，chainid的逻辑已经忽略
                let mut commonsig = CommonSignature::from_vec(&signature.to_vec());
                let v_u64 = Secp256Signature::adjust_v_value(commonsig.v[0] as u64);
                commonsig.v = v_u64.to_be_bytes().to_vec();
                Ok(commonsig)
            }
            Err(e) => {
                kisserr!(KissErrKind::ESign, "wedpr failt {:?}", e)
            }
        }
    }
}

//----------------------------------------------
// parity_crypto实现的secp256签名
impl CommonSignerSecp256 {
    pub fn from_hex_key(&mut self, hextext: &str) {
        self.private_key = hex::decode(hextext.as_bytes()).unwrap();
        //self.keypair = KeyPair::from_secret_slice(binkey.as_slice()).unwrap();
    }
    pub fn keypair(&self) -> Result<KeyPair, parity_crypto::publickey::Error> {
        KeyPair::from_secret_slice(self.private_key.as_slice())
    }
}
*/
//---------------------------------------------
///基于wedpr的Secp256库实现签名
#[derive(Default, Debug, Clone)]
pub struct CommonSignerWeDPR_Secp256 {
    //pub  private_key:Vec<u8>,
    pub account: BcosAccount,
    pub signer: WedprSecp256k1Recover,
}

//unsafe impl Send for CommonSignerWeDPR_Secp256{}
//unsafe impl Sync for CommonSignerWeDPR_Secp256{}
impl ICommonSigner for CommonSignerWeDPR_Secp256 {
    fn sign(&self, data: Vec<u8>) -> Result<CommonSignature, KissError> {
        printlnex!("current signer is :{:?}", self);
        //let signature = WedprSecp256k1Recover::default();
        let msg_signature = self.signer.sign(&self.account.privkey, &data);
        match msg_signature {
            Ok(signature) => {
                //采用secp256签名后，直接调整v的值，如果是0,1则+27 ，4不变，chainid的逻辑已经忽略
                let mut commonsig = CommonSignature::from_vec(&signature);
                let v_u64 = Secp256Signature::adjust_v_value(commonsig.v[0] as u64);
                commonsig.v = v_u64.to_be_bytes().to_vec();
                printlnex!("after adjust value v is {:?}", commonsig.v);
                Ok(commonsig)
            }
            Err(e) => {
                kisserr!(KissErrKind::ESign, "wedpr failt {:?}", e)
            }
        }
    }
}

impl CommonSignerWeDPR_Secp256 {
    fn new(key: &Vec<u8>) -> CommonSignerWeDPR_Secp256 {
        let mut signer = CommonSignerWeDPR_Secp256::default();
        signer.key_from_bytes(&key.as_slice());
        signer
    }
    pub fn key_from_bytes(&mut self, keybytes: &[u8]) {
        let acc = EcdsaAccountUtil::default().from_privkey_bytes(&keybytes.to_vec());
        self.account = acc.unwrap();
    }
    pub fn key_from_hexstr(&mut self, hextext: &str) {
        let prikey_fix = hex::decode(hextext);
        self.key_from_bytes(prikey_fix.unwrap().as_slice())
    }
}

//---------------------------------------------
///基于wedpr的国密sm2库实现签名
#[derive(Default, Debug, Clone)]
pub struct CommonSignerWeDPR_SM2 {
    pub account: BcosAccount,
    pub signer: WedprSm2p256v1,
}

lazy_static! {
    static ref SM2SIGHER: WedprSm2p256v1 = WedprSm2p256v1::default();
}
//unsafe impl Send for CommonSignerWeDPR_SM2{}
//unsafe impl Sync for CommonSignerWeDPR_SM2{}
impl ICommonSigner for CommonSignerWeDPR_SM2 {
    fn sign(&self, data: Vec<u8>) -> Result<CommonSignature, KissError> {
        //let SM2SIGHER1:WedprSm2p256v1 = WedprSm2p256v1::default();
        let start = time::now();
        let result = SM2SIGHER.sign(&self.account.privkey, &data);
        let end = time::now();
        printlnex!("sign data use time {:?}", (end - start));
        match result {
            Ok(s) => {
                let mut commonsig = CommonSignature::from_vec(&s);
                commonsig.v = self.account.pubkey.clone(); //国密版本直接设置公钥
                Ok(commonsig)
            }
            Err(e) => {
                kisserr!(KissErrKind::ESign, "gm sign error {:?}", e)
            }
        }
    }
}

impl CommonSignerWeDPR_SM2 {
    pub fn new(key: Vec<u8>) -> CommonSignerWeDPR_SM2 {
        let mut signer = CommonSignerWeDPR_SM2::default();
        signer.key_from_bytes(key.as_slice());
        signer
    }
    pub fn key_from_bytes(&mut self, keybytes: &[u8]) {
        self.account = GMAccountUtil::default()
            .from_privkey_bytes(&keybytes.to_vec())
            .unwrap();
    }
    pub fn key_from_hexstr(&mut self, hextext: &str) {
        let prikey_fix = hex::decode(hextext).unwrap();
        self.key_from_bytes(&prikey_fix)
    }
}

static demokeyhex: &str = "82dcd33c98a23d5d06f9331554e14ab4044a1d71b169b7a38b61c214f0690f80";

pub fn test_common_sign() {
    //let mut ecdsasigner: CommonSignerSecp256 = CommonSignerSecp256::default();
    let mut wedprsigner: CommonSignerWeDPR_Secp256 = CommonSignerWeDPR_Secp256::default();
    let data = keccak_hash::keccak(Vec::from("abcdefg"));
    // ecdsasigner.from_hex_key(demokeyhex);
    wedprsigner.key_from_hexstr(demokeyhex);

    // let mut signer: &dyn ICommonSigner = &ecdsasigner;
    // let s1 = signer.sign(Vec::from(data.as_bytes())).unwrap();
    let signer = &wedprsigner;
    let s2 = signer.sign(Vec::from(data.as_bytes())).unwrap();
    //wedpr转公钥使用了带压缩支持的算法，前面加04是为了标注这个公钥是没有压缩的，64字节的公钥，如果是压缩的33字节公钥前面会是03
    let recover = wedprsigner
        .signer
        .recover_public_key(data.as_bytes(), s2.to_vec().as_slice())
        .unwrap();
    println!(
        "recover by wedpr ,pubkey len{},{:?}",
        &recover.len(),
        &recover
    );
    let sp = Secp256Signature::to_electrum(&s2.to_vec());
    /*
     let sig = ParityEcdsaSignature::from_electrum(sp.as_slice());
     let recoverresult = publickey::recover(&sig, &data).unwrap();
     println!(
     "recover by ecdsa ,pubkey len {}, {:?}",
     recoverresult.as_bytes().len(),
     recoverresult.as_bytes()
     );*/

    let s = CommonSignature::from_vec(&s2.to_vec());

    println!("r={:?},s={:?},v={:?}", s.r, s.s, s.v);
}

pub fn test_gm_sign() {
    let mut sm2signer = CommonSignerWeDPR_SM2::default();
    sm2signer.key_from_hexstr(demokeyhex);

    let signer: &dyn ICommonSigner = &sm2signer;
    let data = "1234567890";
    let signresult = signer.sign(data.as_bytes().to_vec());
    println!("GM Sign result = {:?}", &signresult);
    let signresult1 = signer.sign(data.as_bytes().to_vec());
    let sig = signresult.unwrap();
    println!("account detail: {:?}", sm2signer.account.to_hexdetail());
    println!("GM Sign Hex = {:?}", hex::encode(&sig.to_vec().as_slice()));

    let sigsm2 = WEDPRSM2Signature::bytes_decode(&sig.to_vec().as_slice()).unwrap();
    println!("sm2 sig {:?}", sigsm2);
    println!(
        "sm sig is r:{:?},s{:?},v:{:?}({})",
        hex::encode(&sig.r),
        hex::encode(&sig.s),
        hex::encode(&sig.v),
        &sig.v.len()
    );
}
