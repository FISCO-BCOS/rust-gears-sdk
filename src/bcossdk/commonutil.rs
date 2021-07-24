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
    non_upper_case_globals,
    overflowing_literals,
    unused_variables,
    unused_assignments
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OSKind {
    windows,
    linux,
    macos,
    unknow,
}

pub fn detect_os() -> OSKind {
    if cfg!(target_os = "windows") {
        return OSKind::windows;
    }
    if cfg!(target_os = "linux") {
        return OSKind::linux;
    }
    if cfg!(target_os = "macos") {
        return OSKind::macos;
    }
    return OSKind::linux;
}

pub fn is_windows() -> bool {
    if detect_os() == OSKind::windows {
        return true;
    }
    return false;
}
