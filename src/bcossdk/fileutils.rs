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
use std::fs::File;
use std::io::{Read, Write};


use crate::bcossdk::kisserror::{KissErrKind, KissError};
use std::path::Path;

///封装略显繁琐的读文件操作
pub fn read_all(fullpath: &str) -> Result<Vec<u8>, KissError> {
    let mut file = match File::open(fullpath) {
        Ok(f) => f,
        Err(e) => {
            return kisserr!(
                KissErrKind::EFileOpen,
                "open file {:?} error {:?}",
                fullpath,
                e
            )
        }
    };

    let mut val = Vec::new();
    let readresult = file.read_to_end(&mut val);

    match readresult {
        Ok(s) => Ok(val),
        Err(e) => {
            kisserr!(
                KissErrKind::EFileRead,
                "read file {:?} error {:?}",
                fullpath,
                e
            )
        }
    }
}

pub fn readstring(fullpath: &str) -> Result<String, KissError> {
    let data = read_all(fullpath)?;
    let res = String::from_utf8(data);
    match res {
        Ok(s) => Ok(s),
        Err(e) => {
            kisserr!(
                KissErrKind::EFormat,
                "read file {:?},error {:?}",
                fullpath,
                e
            )
        }
    }
}

pub fn writestring(fullpath: &str, str: String) -> Result<(), KissError> {
    write_all(fullpath, str.into_bytes())
}
///封装略显繁琐的写文件操作
pub fn write_all(fullpath: &str, data: Vec<u8>) -> Result<(), KissError> {
    let mut file = match File::create(fullpath) {
        Ok(f) => f,
        Err(e) => {
            return kisserr!(KissErrKind::Error, "open file {:?} error {:?}", fullpath, e);
        }
    };

    let writeresult = file.write_all(data.as_slice());
    match writeresult {
        Ok(s) => Ok(()),
        Err(e) => {
            kisserr!(KissErrKind::Error, "read file {:?} error {:?}", fullpath, e)
        }
    }
}

pub fn is_file_exist(fullpath:&str)->bool
{
    let res =  Path::new(fullpath).exists();
    return res;
}
