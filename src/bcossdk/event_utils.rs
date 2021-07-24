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
use crate::bcossdk::commonhash::{CommonHash, HashType};
use crate::bcossdk::kisserror::{KissErrKind, KissError};
use ethabi::param_type::Writer;
use ethabi::{Event, EventParam, Hash, Log, LogParam, ParamType, RawLog, Token};
use std::collections::HashMap;
/*
此文件的实现参考了https://docs.rs/ethabi，https://github.com/rust-ethereum/ethabi
该项目采用Apache许可
由于其部分实现是私有的，所以在这里参考原代码进行修改
*/
/*由于event解析过程中，有用到hash的地方，在适配国密时需要改为国密，所以单独将event解析的代码独立出来*/
/*tip: 依次推理，在相关的一些库使用时，如果用到hash算法，一定要注意国密和非国密的实现*/
fn convert_topic_param_type(kind: &ParamType) -> ParamType {
    match kind {
        ParamType::String
        | ParamType::Bytes
        | ParamType::Array(_)
        | ParamType::FixedArray(_, _)
        | ParamType::Tuple(_) => ParamType::FixedBytes(32),
        _ => kind.clone(),
    }
}

fn params_names(event: &Event) -> Vec<String> {
    event.inputs.iter().map(|p| p.name.clone()).collect()
}

fn indexed_params(event: &Event, indexed: bool) -> Vec<EventParam> {
    event
        .inputs
        .iter()
        .filter(|p| p.indexed == indexed)
        .cloned()
        .collect()
}
pub fn event_signature(e: &Event, hashtype: &HashType) -> Hash {
    let param_typs: Vec<ParamType> = e.inputs.iter().map(|p| p.kind.clone()).collect();
    let paramsarray: &[ParamType] = param_typs.as_slice();
    let types = paramsarray
        .iter()
        .map(Writer::write)
        .collect::<Vec<String>>()
        .join(",");
    let data: Vec<u8> = From::from(format!("{}({})", e.name, types).as_str());
    let hashbytes = CommonHash::hash(&data, hashtype);
    let hash: Hash = Hash::from_slice(hashbytes.as_slice());
    hash
}

/// Parses `RawLog` and retrieves all log params from it.
pub fn parse_log(event: &Event, log: RawLog, hashtype: &HashType) -> Result<Log, KissError> {
    let topics = log.topics;
    let data = log.data;
    let topics_len = topics.len();
    // obtains all params info
    let topic_params = indexed_params(event, true);
    let data_params = indexed_params(event, false);
    // then take first topic if event is not anonymous
    let to_skip = if event.anonymous {
        0
    } else {
        // verify
        let eventsig_topic = topics
            .get(0)
            .ok_or(KissError::new(KissErrKind::EFormat, format!("miss sig")))?;
        let evsig = event_signature(event, hashtype);
        if eventsig_topic != &evsig {
            return kisserr!(KissErrKind::Error, "Invalidata wrong signature");
        }
        1
    };

    let topic_types = topic_params
        .iter()
        .map(|p| convert_topic_param_type(&p.kind))
        .collect::<Vec<ParamType>>();

    let flat_topics = topics
        .into_iter()
        .skip(to_skip)
        .flat_map(|t| t.as_ref().to_vec())
        .collect::<Vec<u8>>();

    let topic_tokens_res = ethabi::decode(&topic_types, &flat_topics);
    let topic_tokens = match topic_tokens_res {
        Ok(tt) => tt,
        Err(e) => return kisserr!(KissErrKind::EFormat, "decode topic_types error {:?}", e),
    };

    // topic may be only a 32 bytes encoded token
    if topic_tokens.len() != topics_len - to_skip {
        return kisserr!(KissErrKind::Error, "Invalidata wrong signature");
    }

    let topics_named_tokens = topic_params
        .into_iter()
        .map(|p| p.name)
        .zip(topic_tokens.into_iter());

    let data_types = data_params
        .iter()
        .map(|p| p.kind.clone())
        .collect::<Vec<ParamType>>();

    let data_tokens_res = ethabi::decode(&data_types, &data);
    let data_tokens = match data_tokens_res {
        Ok(dt) => dt,
        Err(e) => return kisserr!(KissErrKind::EFormat, "decode data_types error {:?}", e),
    };

    let data_named_tokens = data_params
        .into_iter()
        .map(|p| p.name)
        .zip(data_tokens.into_iter());

    let named_tokens = topics_named_tokens
        .chain(data_named_tokens)
        .collect::<HashMap<String, Token>>();

    let param_names = params_names(event);
    let decoded_params = param_names
        .into_iter()
        .map(|name| LogParam {
            name: name.clone(),
            value: named_tokens[&name].clone(),
        })
        .collect();

    let result = Log {
        params: decoded_params,
    };

    Ok(result)
}
