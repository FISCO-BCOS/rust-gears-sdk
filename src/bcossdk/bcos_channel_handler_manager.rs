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

use crate::bcossdk::channelpack::{ChannelPack};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait IChannelPushHandlerFacade
{
    ///传入一个packet,由实现者去自行处理
    fn handle(self:&Self,pack:&ChannelPack);
}
pub type HANDLE_FACADE_OBJ = Arc<Mutex<dyn IChannelPushHandlerFacade+Send+Sync>>;
pub type CHANNEL_HANDLE_MAP = HashMap<u16, HANDLE_FACADE_OBJ>;
pub struct ChannelPushHandlerManager
{
    dispatch_register: CHANNEL_HANDLE_MAP
}

impl ChannelPushHandlerManager {
    pub fn default()-> Self
    {
        ChannelPushHandlerManager
        {
            dispatch_register:CHANNEL_HANDLE_MAP::new()
        }
    }

    pub fn set_handle(&mut self,handletype:&u16,handle:HANDLE_FACADE_OBJ)
    {
        println!("set handler {}",handletype);
        match self.dispatch_register.get(handletype){
            Some(v)=>{return},
            None=>{}
        }
        self.dispatch_register.insert(handletype.clone(),handle);
    }

    pub fn remove_handler(&mut self,handletype:&u16){
        //let mut m = self.dispatch_register.lock().unwrap();
        self.dispatch_register.remove(&handletype);
    }
    pub fn get_handle(&self,handletype:&u16)->Option<&HANDLE_FACADE_OBJ>{
       // let m = self.dispatch_register.lock().unwrap();
        println!("get handler {}",handletype);
        let handle = self.dispatch_register.get(handletype);
        handle
    }

    pub fn count_handler(&self)->usize{
        self.dispatch_register.len()
    }

}