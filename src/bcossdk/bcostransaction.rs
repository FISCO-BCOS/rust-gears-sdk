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
use std::str::FromStr;

use ethereum_types::{Address, H256, H512, U256};
use rlp::{self, DecoderError, Rlp, RlpStream};

use crate::bcossdk::accountutil::{EcdsaAccountUtil, IBcosAccountUtil};
use crate::bcossdk::commonhash::{CommonHash, HashType};
use crate::bcossdk::commonsigner::{CommonSignature, CommonSignerWeDPR_Secp256, ICommonSigner};
use crate::bcossdk::kisserror::KissError;
use std::convert::TryInto;

///fisco bcos的交易结构，重点关注random_id,block_limit,chain_id,grou_id
#[derive(Debug, Clone)]
pub struct BcosTransaction {
    pub random_id: U256,
    pub gas_price: U256,
    pub gas_limit: U256,
    pub block_limit: U256,
    pub to_address: Vec<u8>,
    pub value: U256,
    pub data: Vec<u8>,
    pub fisco_chain_id: U256,
    pub group_id: U256,
    pub extra_data: Vec<u8>,
    pub hashtype: HashType, //由调用者制定hash函数的类型
}

impl BcosTransaction {
    pub fn default() -> BcosTransaction {
        BcosTransaction {
            random_id: U256::default(),
            gas_price: U256::default(),
            gas_limit: U256::default(),
            block_limit: U256::default(),
            to_address: vec![],
            value: U256::default(),
            data: vec![],
            fisco_chain_id: U256::default(),
            group_id: U256::default(),
            extra_data: vec![],
            hashtype: HashType::Unknow, //由调用者制定hash函数的类型
        }
    }

    pub fn rlp_append_tx_elements(&self, stream: &mut RlpStream) {
        stream.append(&self.random_id);
        stream.append(&self.gas_price);
        stream.append(&self.gas_limit);
        stream.append(&self.block_limit);
        stream.append(&self.to_address);
        stream.append(&self.value);
        stream.append(&self.data);
        stream.append(&self.fisco_chain_id);
        stream.append(&self.group_id);
        stream.append(&self.extra_data);
    }

    pub fn encode(&self) -> Vec<u8> {
        let listsize = 10;
        let mut stream = RlpStream::new();
        stream.begin_list(listsize);
        self.rlp_append_tx_elements(&mut stream);
        let encodebytes: Vec<u8> = stream.drain();
        encodebytes
    }

    ///把交易内容encode成rlp，进行hash，用于签名，注意这不是发送交易后得到的txhash
    pub fn hash(&self) -> H256 {
        CommonHash::hash_to_h256(&self.encode(), &self.hashtype)
    }
    ///把字节打入rlp，然后解析出来tx
    pub fn decode_bytes(txbyes: &[u8]) -> Result<BcosTransaction, DecoderError> {
        BcosTransaction::decode_rlp(&Rlp::new(txbyes), 0)
    }

    pub fn decode_rlp(rlp: &Rlp, start_pos: usize) -> Result<BcosTransaction, DecoderError> {
        Ok(BcosTransaction {
            random_id: rlp.val_at(start_pos)?,
            gas_price: rlp.val_at(start_pos + 1)?,
            gas_limit: rlp.val_at(start_pos + 2)?,
            block_limit: rlp.val_at(start_pos + 3)?,
            to_address: rlp.val_at(start_pos + 4)?,
            value: rlp.val_at(start_pos + 5)?,
            data: rlp.val_at(start_pos + 6)?,
            fisco_chain_id: rlp.val_at(start_pos + 7)?,
            group_id: rlp.val_at(start_pos + 8)?,
            extra_data: rlp.val_at(start_pos + 9)?,
            hashtype: HashType::Unknow,
        })
    }
}

///携带签名信息的完整交易内容，用于发送给节点
#[derive(Debug, Clone)]
pub struct BcosTransactionWithSig {
    pub transaction: BcosTransaction,
    pub signature: CommonSignature,
    pub is_signed: bool,
}

impl BcosTransactionWithSig {
    /*携带签名的encode*/
    pub fn encode(&self) -> Vec<u8> {
        let mut stream = RlpStream::new();
        stream.begin_list(13);
        self.transaction.rlp_append_tx_elements(&mut stream);
        self.rlp_append_signature(&mut stream);
        stream.drain()
    }

    pub fn decode_bytes(txbyes: &Vec<u8>) -> Result<BcosTransactionWithSig, DecoderError> {
        BcosTransactionWithSig::decode_rlp(&Rlp::new(&txbyes.as_slice()))
    }

    pub fn rlp_append_signature(&self, stream: &mut RlpStream) {
        let s: H256 = H256::default();
        printlnex!(
            "v:{},r:{},s:{}",
            &self.signature.v.len(),
            &self.signature.r.len(),
            &self.signature.s.len()
        );
        // printlnex!("v{:?},r{:?},s{:?}",&self.signature.v,&self.signature.r,&self.signature.s);
        if self.signature.v.len() == 8
        // ecdsa
        {
            printlnex!("is a ecdsa sig {:?}", self.signature.v);
            let u64v: u64 =
                u64::from_be_bytes(self.signature.v.as_slice()[0..8].try_into().unwrap());
            stream.append(&u64v);
        } else {
            let h512v = H512::from_slice(self.signature.v.as_slice());
            printlnex!("append v {:?}", hex::encode(h512v.as_bytes()));
            stream.append(&h512v);
        }
        let h256r = H256::from_slice(&self.signature.r.as_slice());
        stream.append(&h256r);
        let h256s: H256 = H256::from_slice(&self.signature.s.as_slice());
        stream.append(&h256s);
    }

    pub fn decode_rlp(d: &Rlp) -> Result<BcosTransactionWithSig, DecoderError> {
        if d.item_count()? < 10 {
            return Err(DecoderError::RlpIncorrectListLen);
        }
        //let hash = keccak(d.as_raw());
        let transaction = BcosTransaction::decode_rlp(d, 0).unwrap();

        let mut is_signed = false;
        let mut signature = CommonSignature::default();
        if d.item_count().unwrap() == 13 {
            //从index 10开始，是签名 v,r,s
            let v: Vec<u8> = d.val_at(10)?;
            let r: Vec<u8> = d.val_at(11)?;
            let s: Vec<u8> = d.val_at(12)?;

            signature = CommonSignature { v: v, r: r, s: s };
            is_signed = true;
        }
        Ok(BcosTransactionWithSig {
            transaction,
            signature,
            is_signed,
        }) //Ok
    }

    ///传入字节类型的原始私钥，对交易进行签名
    pub fn sign(
        signer: &dyn ICommonSigner,
        tx: &BcosTransaction,
    ) -> Result<BcosTransactionWithSig, KissError> {
        //以上这两行后续修改为支持国密
        let sig_result = signer.sign(tx.hash().as_bytes().to_vec());
        printlnex!("sign tx: {:?}", sig_result);
        match sig_result {
            Ok(sig) => {
                printlnex!("sign tx ok : {:?}", &sig.detail());
                let signtx = BcosTransactionWithSig {
                    transaction: tx.clone(),
                    signature: sig,
                    is_signed: true,
                };
                Ok(signtx)
            }
            Err(e) => Err(e),
        }
    }
}

fn test_decode_tx_from_str(tx_data: &str) {
    let tx = BcosTransaction::decode_bytes(hex::decode(tx_data).unwrap().as_slice()).unwrap();
}

///地址转码的helper,当传入地址为空时,直接反回空数组,否则转一下Address(其实就是hexdecode)
pub fn encode_address(addr: &str) -> Vec<u8> {
    if addr.trim().len() == 0 {
        return Vec::from(addr);
    }

    let addrcheck = addr.trim().trim_start_matches("0x");
    printlnex!("addr checked: {}", addrcheck);
    let addr = Address::from_str(addrcheck).unwrap();
    return Vec::from(addr.as_bytes());
}

///一个简单的工具方法，先保留，严格来说传入的参数是不够的。
pub fn encode_raw_transaction(
    to_address: &String,
    rawdata: &String,
    key: &Vec<u8>,
    hashtype: HashType,
) -> Vec<u8> {
    let randid: u64 = rand::random();
    println!("raw tx randid = {}", randid);
    let tx = BcosTransaction {
        to_address: encode_address(to_address),
        random_id: U256::from(randid),
        gas_price: U256::from(30000000),
        gas_limit: U256::from(30000000),
        block_limit: U256::from(501),
        value: U256::from(0),
        data: hex::decode(rawdata.as_str()).unwrap(),
        fisco_chain_id: U256::from(1),
        group_id: U256::from(1),
        extra_data: b"".to_vec(),
        hashtype,
    };
    let txencodedata = tx.encode();
    //println!("tx encode {:?}", txencodedata);
    let mut signer = CommonSignerWeDPR_Secp256::default();
    signer.key_from_bytes(key);
    let t = BcosTransactionWithSig::sign(&signer, &tx);
    let tx = t.unwrap().encode();
    tx
}

///测试代码入口
pub fn test_sign_tx() {
    let key = EcdsaAccountUtil::default().create_random();
    let set_input = "4ed3885e0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000b3132333437383930616263000000000000000000000000000000000000000000";
    //let addr = Address::from_str("40034be5fd46006238c04c2cedfe92dbddbdb651").unwrap();
    //let addr = String::from("");
    let addr = String::from("40034be5fd46006238c04c2cedfe92dbddbdb651");
    let raw_tx_encode = encode_raw_transaction(
        &addr,
        &String::from(set_input),
        &key.privkey,
        HashType::WEDPR_KECCAK,
    );
    println!("raw_tx_encode {:?}", raw_tx_encode);
    let tx = BcosTransactionWithSig::decode_bytes(&raw_tx_encode).unwrap();
    println!("after decode SignedBcosTransaction");
    println!(
        "to address {:?}",
        Address::from_slice(tx.transaction.to_address.as_slice())
    );
    println!("to address {:?}", hex::encode(tx.transaction.to_address));
}

pub fn test_decode_tx() {
    //let datahex: &str = "f864808504a817c800825208943535353535353535353535353535353535353535808025a0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116da0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d";
    let datahex = "f8ea2a820bb882c3503294d46e8dd67c5d32be8058bb8eb970870f0724456701b8c834656433383835653030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030323030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303034333133323333333430303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030300101836162631ca0ecf1aa38a05271a7d0c90a6f85b558458620812244d04bebbe94e093a45404faa05e4b294198175184cb4b35d109d00457300afefee5efc3cadf59718bc27bfd47";
    let rawdata = hex::encode("abcdefg");
    let randid = 209;

    let tx = BcosTransaction {
        to_address: Vec::from(
            Address::from_str("40034be5fd46006238c04c2cedfe92dbddbdb651")
                .unwrap()
                .as_bytes(),
        ),
        random_id: U256::from(randid),
        gas_price: U256::from(30000000),
        gas_limit: U256::from(30000000),
        block_limit: U256::from(501),
        value: U256::from(0),
        data: hex::decode(rawdata.as_str()).unwrap(),
        fisco_chain_id: U256::from(1),
        group_id: U256::from(1),
        extra_data: b"".to_vec(),
        hashtype: HashType::WEDPR_KECCAK,
    };
    let txencodedata = tx.encode();
    //println!("hash tx {:?}",tx.hash());
    //println!("tx hex: {:?}",&hex::encode(datahex.as_slice()));
    //let hexstr = hex::encode(datahex.as_slice()).as_str();
    //test_decode_tx_from_str(txencodedata.as_slice().to_hex().as_str());

    test_sign_tx();
}
