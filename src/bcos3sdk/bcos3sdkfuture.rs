/*
为sdk实现同步等待response的方式

基本原理是通过mpsc的sender/receiver，当收到回调时fire一个消息到send队列里，同步等待的receiver即可得到消息

可以将此future的对象指针直接传给sdk的回调context，也可以本地全局化一个mapping之类的，把key传给sdk，在回调时用key去找到这个future对象

这里的实现仅供参考。在多线程、异步编程时，可以根据sdk的callback定义，自行实现灵活的回调处理

*/

use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use serde_json::Value as JsonValue;

use crate::bcos3sdk::bcos3sdkresponse::{bcos_sdk_c_struct_response, Bcos3SDKResponse};
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use crate::kisserrcode;

#[repr(C)]
#[derive(Debug)]
pub struct Bcos3SDKFuture {
    pub seq: u64,
    pub name: String,
    pub desc: String,
    pub timeout: u64, //等待返回的超时，默认为5秒，可以动态修改或配置化
    // 描述
    pub tx: Sender<Bcos3SDKResponse>,
    pub rx: Receiver<Bcos3SDKResponse>,
}

static gSeq: AtomicU64 = AtomicU64::new(0);

impl Bcos3SDKFuture {
    pub fn next_seq() -> u64 {
        gSeq.fetch_add(1, Ordering::Relaxed)
    }
    //使用mpsc::channel组件实现异步回调的等待
    pub fn create(seq: u64, name: &str, desc: &str) -> Self {
        let (tx, rx) = mpsc::channel();

        let future_context = Bcos3SDKFuture {
            seq: seq,
            name: name.to_string(),
            desc: desc.to_string(),
            timeout: 5,
            tx: tx,
            rx: rx,
        };
        future_context
    }

    //和C结构体指针互转，裸指针逻辑
    //* C语言sdk要求传入指针，而回调的生命周期范围是由C语言SDK决定的
    //* 使用这个指针时，必须保证指针指向的对象 活着，没有被释放，没有被borrow掉，
    //* 保持简单，不要用box into_raw/from_raw这些“智能指针”，因为其内存管理可能会不符合C的预期，比如用一次再用就是double free
    //* 如果自行alloc，比如在堆上，要记得释放
    //* 如果要在rust里跨线程，要注意跨线程变量的生命周期范围，搞清楚内存模型
    //* 如果要放到容器里，也要注意容器的clone,borrow等细节
    //* 改成string,int什么的，效果也是一样的，在内存管理和生命周期管理方面没区别
    //* 唯一比较安全的可能是用u64，指针本身就是u64值传递，然后再用这个u64值来查找对应关系
    //* 总之，unsafe，小心！
    pub fn to_c_ptr(c: &Self) -> *const c_void {
        return c as *const Bcos3SDKFuture as *const c_void;
    }

    pub fn from_c_ptr(ptr: *const c_void) -> *const Bcos3SDKFuture {
        return ptr as *const Bcos3SDKFuture;
    }

    pub fn display(&self) {
        println!(
            ">>>> context data:{},[{}],[{}]",
            self.seq, self.name, self.desc
        );
    }

    //在rust里，要给c的回调传入method, not a field，跟python sdk的实现对比，和python可以将某个对象的方法地址传给c不同了。
    //所以在这个回调方法里，reponse 里的context给的是future本身，以实例化该对象后，进行异步应答（mpsc::channel)

    pub extern "C" fn bcos_callback(resp: *const bcos_sdk_c_struct_response) {
        unsafe {
            let response = Bcos3SDKResponse::from_callback(resp);
            if !(*resp).context.is_null() {
                //强行的指针转换成struct指针，必须在生命周期内，其实是挺危险的。
                (*Bcos3SDKFuture::from_c_ptr((*resp).context)).fire(&response)
            }
        }
    }
    pub fn fire(&self, resp: &Bcos3SDKResponse) {
        let res = self.tx.send(resp.clone());
    }

    pub fn wait(&self) -> Result<Bcos3SDKResponse, KissError> {
        //超时默认设定为5秒，可以修改
        let res = self
            .rx
            .recv_timeout(std::time::Duration::from_secs(self.timeout));
        //println!("wait res {:?}",&res);
        match res {
            Ok(v) => {
                return Ok(v);
            }
            Err(e) => {
                return kisserrcode!(KissErrKind::ETimeout, -1, "timeout");
            }
        }
    }
    pub fn wait_result(&self) -> Result<JsonValue, KissError> {
        let response = self.wait()?;
        response.get_result()
    }
}
