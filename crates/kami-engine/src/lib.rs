//! # kami-engine
//!
//! WASM engine powered by Wasmtime with Component Model support.
//! Handles component compilation, instance creation, WASI linking,
//! and async execution.

pub mod bindings;
pub mod component;
pub mod error;
pub mod instance;
pub mod memory;
pub mod state;

pub use bindings::{call_describe, call_run, instantiate_tool};
pub use component::{
    call_tool_run, create_linker, instantiate_component, load_component, load_component_from_file,
};
pub use error::EngineError;
pub use instance::{create_engine, create_store, set_epoch_deadline, InstanceConfig};
pub use memory::MemoryStats;
pub use state::HostState;
