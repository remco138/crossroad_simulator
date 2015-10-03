#![feature(custom_derive, plugin, custom_attribute)]
#![plugin(serde_macros)]
#![allow(dead_code, unused_variables, unused_imports)]

extern crate serde_json;
extern crate serde;

pub mod traffic_protocol;
pub mod error;

