// prevent linter warnings for these scenarios  //this should prob be set up in its own clippy.toml config file in the crate root
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::assign_op_pattern)]    //allow x = x + y, instead of x += y
#![allow(clippy::if_same_then_else)]
#![allow(clippy::match_same_arms)]  //idk,double check if we want this one...
#![allow(clippy::bool_to_int_with_if)]  //idk, double check if we want this one...
//#![warn(unused_results)]
#![allow(clippy::cast_possible_truncation)]

pub mod application;
pub mod mode;
pub mod action;
pub mod history;
pub mod position;
pub mod buffer;
pub mod range;
pub mod selection;
pub mod selection2d;
pub mod selections;
pub mod display_area;
pub mod keybind;
pub mod ui;
pub mod config;
pub mod tutorial;
pub mod mode_stack;

pub mod utilities;
//#[cfg(test)] mod tests;
