#![feature(custom_derive, plugin, custom_attribute, box_syntax, iter_cmp, convert, vec_push_all)]
#![plugin(serde_macros)]
#![allow(dead_code, unused_variables, unused_imports)]

extern crate serde_json;
extern crate serde;
extern crate time;

pub mod traffic_protocol;
pub mod kruispunt;
pub mod error;

