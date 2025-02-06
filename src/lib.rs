/*
 * Copyright (C) 2019 Intel Corporation. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

 #![doc = include_str!("../README.md")]

#[cfg(feature = "macros")]
pub extern crate wamr_macros;

use std::{error, fmt, io};

pub use wamr_sys as sys;
pub use wamr_macros::generate_host_function;

pub mod function;
mod helper;
pub mod host_function;
pub mod instance;
pub mod module;
pub mod runtime;
pub mod value;
pub mod wasi_context;

#[derive(Debug)]
pub struct ExecError {
    pub message: String,
    pub exit_code: u32,
}

/// all kinds of exceptions raised by WAMR
#[derive(Debug)]
pub enum RuntimeError {
    NotImplemented,
    /// Runtime initialization error.
    InitializationFailure,
    /// file operation error. usually while loading(compilation) a .wasm
    WasmFileFSError(std::io::Error),
    /// A compilation error. usually means that the .wasm file is invalid
    CompilationError(String),
    /// instantiation failure
    InstantiationFailure(String),
    /// Error during execute wasm functions
    ExecutionError(ExecError),
    /// usually returns by `find_export_func()`
    FunctionNotFound,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeError::NotImplemented => write!(f, "Not implemented"),
            RuntimeError::InitializationFailure => write!(f, "Runtime initialization failure"),
            RuntimeError::WasmFileFSError(e) => write!(f, "Wasm file operation error: {}", e),
            RuntimeError::CompilationError(e) => write!(f, "Wasm compilation error: {}", e),
            RuntimeError::InstantiationFailure(e) => write!(f, "Wasm instantiation failure: {}", e),
            RuntimeError::ExecutionError(info) => write!(
                f,
                "Wasm execution error: {} and {}",
                info.message, info.exit_code
            ),
            RuntimeError::FunctionNotFound => write!(f, "Function not found"),
        }
    }
}

impl error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            RuntimeError::WasmFileFSError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for RuntimeError {
    fn from(e: io::Error) -> Self {
        RuntimeError::WasmFileFSError(e)
    }
}
