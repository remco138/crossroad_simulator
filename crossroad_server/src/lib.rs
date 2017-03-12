#![feature(core_intrinsics, custom_derive, plugin, custom_attribute, box_syntax)]
#![allow(unused_variables, unused_imports)]

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate time;
#[macro_use] extern crate itertools;
extern crate permutohedron;

pub mod traffic_protocol;
pub mod traffic_controls;
pub mod crossroad;
pub mod default_crossroad;
pub mod error;
pub mod cartesian;
pub mod signal_group;

trait BoolToOpt {
    fn to_opt(&self) -> Option<()>;
}

impl BoolToOpt for bool {
    fn to_opt(&self) -> Option<()> {
        if *self { Some(()) } else { None }
    }
}
