/*
 * Copyright (C) 2023 Liquid Reply GmbH. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

extern crate bindgen;
extern crate cmake;

use cmake::Config;
use std::{env, path::PathBuf};

fn main() {
    let wamr_root = env::current_dir().unwrap();
    let wamr_root = wamr_root.join("wasm-micro-runtime");
    assert!(wamr_root.exists());

    let is_espidf = env::var("CARGO_CFG_TARGET_OS").unwrap() != "espidf";

    if is_espidf {
        let enable_llvm_jit = if cfg!(feature = "llvmjit") { "1" } else { "0" };
        // TODO: define LLVM_DIR
        let dst = Config::new(&wamr_root)
            // running mode
            .define("WAMR_BUILD_AOT", "1")
            .define("WAMR_BUILD_INTERP", "1")
            .define("WAMR_BUILD_FAST_INTERP", "1")
            .define("WAMR_BUILD_JIT", enable_llvm_jit)
            // mvp
            .define("WAMR_BUILD_BULK_MEMORY", "1")
            .define("WAMR_BUILD_REF_TYPES", "1")
            .define("WAMR_BUILD_SIMD", "1")
            // wasi
            .define("WAMR_BUILD_LIBC_WASI", "1")
            // `nostdlib`
            .define("WAMR_BUILD_LIBC_BUILTIN", "0")
            .build_target("iwasm_static")
            .build();

        println!("cargo:rustc-link-search=native={}/build", dst.display());
        println!("cargo:rustc-link-lib=static=vmlib");
    }

    //TODO: support macos?
    if cfg!(feature = "llvmjit") {
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=m");
        println!("cargo:rustc-link-lib=dylib=rt");
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=dylib=z");

        let llvm_dir = wamr_root.join("core/deps/llvm/build");
        assert!(llvm_dir.exists());

        println!("cargo:libdir={}/lib", llvm_dir.display());
        println!("cargo:rustc-link-search=native={}/lib", llvm_dir.display());

        for llvm_lib in &[
            "LLVMAggressiveInstCombine",
            "LLVMAnalysis",
            "LLVMAsmParser",
            "LLVMAsmPrinter",
            "LLVMBitReader",
            "LLVMBitWriter",
            "LLVMCFGuard",
            "LLVMCodeGen",
            "LLVMCoroutines",
            "LLVMCoverage",
            "LLVMDWARFLinker",
            "LLVMDWP",
            "LLVMDebugInfoCodeView",
            "LLVMDebugInfoDWARF",
            "LLVMDebugInfoGSYM",
            "LLVMDebugInfoMSF",
            "LLVMDebugInfoPDB",
            "LLVMDlltoolDriver",
            "LLVMExecutionEngine",
            "LLVMExtensions",
            "LLVMFileCheck",
            "LLVMFrontendOpenACC",
            "LLVMFrontendOpenMP",
            "LLVMFuzzMutate",
            "LLVMGlobalISel",
            "LLVMIRReader",
            "LLVMInstCombine",
            "LLVMInstrumentation",
            "LLVMInterfaceStub",
            "LLVMInterpreter",
            "LLVMJITLink",
            "LLVMLTO",
            "LLVMLibDriver",
            "LLVMLineEditor",
            "LLVMLinker",
            "LLVMMC",
            "LLVMMCA",
            "LLVMMCDisassembler",
            "LLVMMCJIT",
            "LLVMMCParser",
            "LLVMMIRParser",
            "LLVMObjCARCOpts",
            "LLVMObject",
            "LLVMObjectYAML",
            "LLVMOption",
            "LLVMOrcJIT",
            "LLVMOrcShared",
            "LLVMOrcTargetProcess",
            "LLVMPasses",
            "LLVMProfileData",
            "LLVMRuntimeDyld",
            "LLVMScalarOpts",
            "LLVMSelectionDAG",
            "LLVMSymbolize",
            "LLVMTarget",
            "LLVMTextAPI",
            "LLVMTransformUtils",
            "LLVMVectorize",
            "LLVMX86AsmParser",
            "LLVMX86CodeGen",
            "LLVMX86Desc",
            "LLVMX86Disassembler",
            "LLVMX86Info",
            "LLVMXRay",
            "LLVMipo",
        ] {
            println!("cargo:rustc-link-lib=static={}", llvm_lib);
        }
    }

    let wamr_header = wamr_root.join("core/iwasm/include/wasm_export.h");
    assert!(wamr_header.exists());

    let bindings = bindgen::Builder::default()
        .ctypes_prefix("::core::ffi")
        .use_core()
        .header(wamr_header.into_os_string().into_string().unwrap())
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");
}
