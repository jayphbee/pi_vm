use std::sync::Arc;
use std::clone::Clone;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use pi_lib::atom::Atom;
use pi_lib::handler::{Env, GenType, Handler, Args};

use adapter::{JS, JSType};
use pi_vm_impl::push_callback;

/*
* 通道对端
*/
pub enum VMChannelPeer {
    Any,            //任意虚拟机
    VM(Arc<JS>),    //指定虚拟机
}

/*
* 虚拟机通道
*/
pub struct VMChannel {
    src: VMChannelPeer,             //源
    dst: VMChannelPeer,             //目标
    attrs: HashMap<Atom, GenType>,  //属性表
}

impl Env for VMChannel {
    fn get_attr(&self, key: Atom) -> Option<GenType> {
        match self.attrs.get(&key) {
            None => None,
            Some(value) => Some(value.clone()),
        }
    }

    fn set_attr(&mut self, key: Atom, value: GenType) -> Option<GenType> {
        match self.attrs.entry(key) {
            Entry::Occupied(ref mut e) => {
                Some(e.insert(value))
            },
            Entry::Vacant(e) => {
                e.insert(value);
                None
            },
        }
    }

    fn remove_attr(&mut self, key: Atom) -> Option<GenType> {
        self.attrs.remove(&key)
    }
}

impl VMChannel {
    //构建一个虚拟机通道
    pub fn new(src: VMChannelPeer, dst: VMChannelPeer) -> Self {
        VMChannel {
            src: src,
            dst: dst,
            attrs: HashMap::new(),
        }
    }

    //发送消息
    pub fn send(&self, _name: Atom, _msg: Arc<Vec<u8>>) {
        //TODO
        &self.dst;
    }

    //回应请求
    pub fn response(&self, callback: u32, result: Arc<Vec<u8>>) -> bool {
        match self.src {
            VMChannelPeer::VM(ref js) => {
                let args = Box::new(move |vm: Arc<JS>| -> usize {
                    let array = vm.new_uint8_array(result.len() as u32);
					array.from_bytes(result.as_slice());
                    1
                });
                push_callback(js.clone(), callback, args, Atom::from("vm async call response task"));
                true
            },
            _ => false
        }
    }
}

/*
* 虚拟机通道表
*/
pub struct VMChannelMap {
    gray: usize,                                                                                                                    //灰度值
    map: HashMap<Atom, Arc<Handler<A = Arc<Vec<u8>>, B = Vec<JSType>, C = u32, D = (), E = (), F = (), G = (), H = (), HandleResult = ()>>>, //通道表
}

impl VMChannelMap {
    //构建一个虚拟机通道表
    pub fn new(gray: usize) -> Self {
        VMChannelMap {
            gray: gray,
            map: HashMap::new(),
        }
    }

    //获取灰度值
    pub fn get_gray(&self) -> usize {
        self.gray
    }

    //设置灰度值
    pub fn set_gray(&mut self, gray: usize) -> usize {
        let old = self.gray;
        self.gray = gray;
        old
    }

    //获取处理器数量
    pub fn size(&self) -> usize {
        self.map.len()
    }

    //设置指定名称的处理器，返回同名的上一个处理器
    pub fn set(&mut self, name: Atom, handler: Arc<Handler<A = Arc<Vec<u8>>, B = Vec<JSType>, C = u32, D = (), E = (), F = (), G = (), H = (), HandleResult = ()>>) -> Option<Arc<Handler<A = Arc<Vec<u8>>, B = Vec<JSType>, C = u32, D = (), E = (), F = (), G = (), H = (), HandleResult = ()>>> {
        match self.map.entry(name) {
            Entry::Occupied(ref mut e) => {
                Some(e.insert(handler))
            },
            Entry::Vacant(e) => {
                e.insert(handler);
                None
            },
        }
    }

    //移除指定名称的处理器，返回处理器
    pub fn remove(&mut self, name: Atom) -> Option<Arc<Handler<A = Arc<Vec<u8>>, B = Vec<JSType>, C = u32, D = (), E = (), F = (), G = (), H = (), HandleResult = ()>>> {
        self.map.remove(&name)
    }

    //请求
    pub fn request(&self, js: Arc<JS>, name: Atom, msg: Arc<Vec<u8>>, native_objs: Vec<JSType>, callback: u32) -> bool {
        let handler = match self.map.get(&name) {
            None => {
                return false;
            },
            Some(h) => {
                h
            },
        };

        let mut channel = VMChannel::new(VMChannelPeer::VM(js), VMChannelPeer::Any);
        channel.set_attr(Atom::from("_$gray"), GenType::USize(self.gray));
        handler.handle(Arc::new(channel), name, Args::ThreeArgs(msg, native_objs, callback));
        true
    }
}