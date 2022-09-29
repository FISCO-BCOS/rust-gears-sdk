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

use chrono::format::{DelayedFormat, StrftimeItems};
use chrono::Local;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub fn datetime_str() -> String {
    let now = Local::now();
    let fmt = "%Y-%m-%d %H:%M:%S";
    let dft: DelayedFormat<StrftimeItems> = now.format(fmt);
    let str_datetime: String = dft.to_string(); // 2021-01-04 20:02:09
    str_datetime
}

pub fn json_u64(jsonv: &JsonValue, name: &str, defaultvalue: i64) -> i64 {
    let v_option = jsonv.get(name);
    match v_option {
        Some(v) => {
            let u_option = v.as_u64();
            match u_option {
                Some(num) => {
                    return num as i64;
                }
                None => {
                    return defaultvalue;
                }
            }
        }
        None => {
            return defaultvalue;
        }
    }
}

pub fn json_str(jsonv: &JsonValue, name: &str, defaultvalue: &str) -> String {
    let v_option = jsonv.get(name);
    match v_option {
        Some(v) => {
            let s_option = v.as_str();
            match s_option {
                Some(s) => {
                    return s.to_string();
                }
                None => {
                    return defaultvalue.to_string();
                }
            }
        }
        None => {
            return defaultvalue.to_string();
        }
    }
}

pub fn trim_quot(inputstr: &str) -> String {
    let s = inputstr.trim();
    let s = s.trim_start_matches('\'');
    let s = s.trim_end_matches('\'');
    let s = s.trim_start_matches('"');
    let s = s.trim_end_matches('"');

    s.to_string()
}

pub fn get_opt_str(nameopt: &Option<String>) -> String {
    match nameopt {
        Some(v) => {
            return v.clone();
        }
        None => return "".to_string(),
    }
}

///对较为复杂的输入参数，用split分割不可行， split_param支持以下格式
///1,2,3
///['a','b','c']
///['a',('alice',23)] ->tuple，对应合约结构体,内部包含,
///[('alice',23),('bob',28)] -> tuple结构体数组
///’ab,cd' -> 字符串本身包含逗号,
///'ab\\'cd' -> 字符串包含转义符
pub fn split_param(paramstr: &str) -> Vec<String> {
    let mut stopchardict: HashMap<char, char> = HashMap::new();
    stopchardict.insert('(', ')');
    stopchardict.insert('[', ']');
    stopchardict.insert('{', '}');
    stopchardict.insert('\'', '\'');
    stopchardict.insert('"', '"');
    let mut status = 0;
    let mut arrayres: Vec<String> = Vec::new();
    let splitter = ',';
    let mut oldstatus = 0;
    let mut item: String = "".to_string();
    let mut stopchar: char = '\0';
    let mut itemcounter = 0; //为了当前的item，处理了多少个字符，会大于item.len(),因为其中可能有转义字符
    for c in paramstr.chars() {
        //println!("status={},c= [{}],counter={},item= [{}]",status,c,itemcounter,item);
        if c == '\\' {
            oldstatus = status;
            status = 2; //默认纳入下一个字符
            continue;
        }
        if status == 0 {
            if c == splitter {
                // 遇到分隔符,
                //rintln!("splitter");
                arrayres.push(trim_quot(item.as_str()));
                item = "".to_string();
                continue;
            }
            //当 ",',[等出现在当前要处理这一段字符串的首位，如 "abc 则命中",但a"bc，就不管”了，把他当做字符串的一部分
            if stopchardict.contains_key(&c) && itemcounter == 0 {
                stopchar = stopchardict[&c];
                status = 1; // status = 1意思是这一段字符串必须到配对的stopchar才结束
                item.push(c);
                itemcounter += 1;
                //println!("item={}",item);
                continue;
            }
            item.push(c);
            continue;
        }
        if status == 1 {
            //遇到配对的停止符才允许分割，在中间的,不做为分隔符
            item.push(c);
            itemcounter = 0;
            if c == stopchar {
                stopchar = '\0';
                status = 0;
            }
            continue;
        }
        if status == 2 {
            //转义符后面的一个字符直接加入
            status = oldstatus;
            item.push(c);
            itemcounter = 0;
            continue;
        }
    }
    arrayres.push(trim_quot(item.as_str()));
    //println!("split_param result: {:?}",arrayres);
    arrayres
}
