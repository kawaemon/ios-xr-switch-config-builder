//! Change generation engine.
//!
//! This module orchestrates the process of generating IOS XR commands from
//! simplified change input. It follows a compiler-like architecture:
//! 1. Parse change input (input_parser)
//! 2. Validate changes (validator)
//! 3. Plan diff operations (planner)
//! 4. Generate commands (codegen)

pub mod codegen;
pub mod engine;
pub mod input_parser;
pub mod model;
pub mod planner;
pub mod validator;

pub use engine::ChangeEngine;
