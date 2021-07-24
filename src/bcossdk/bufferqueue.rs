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
///简单封装。后续加入线程安全等特性
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BufferQueue {
    pub queue: Vec<u8>,
}

impl BufferQueue {
    pub fn new() -> BufferQueue {
        BufferQueue { queue: Vec::new() }
    }
    ///简单的将书加入缓冲区
    pub fn append(&mut self, newdata: &mut Vec<u8>) {
        self.queue.append(newdata);
    }
    ///从缓冲区的头部去掉n个部分
    pub fn cut(&mut self, pos: usize) {
        self.queue = self.queue.as_slice()[pos..].to_vec();
    }
}

pub fn test_queue() {
    let mut queue = BufferQueue::new();
    let mut v: Vec<u8> = [1, 2, 3, 4, 5].to_vec();
    queue.append(&mut v);
    println!("{:?}", queue);
    let mut v1: Vec<u8> = [6, 7, 8].to_vec();
    queue.append(&mut v1);
    println!("{:?}", queue);
    queue.cut(3);
    println!("{:?}", queue);
}
