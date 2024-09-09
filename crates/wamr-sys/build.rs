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

    println!("cargo:rerun-if-env-changed=WAMR_BUILD_PLATFORM");
    println!("cargo:rerun-if-env-changed=WAMR_SHARED_PLATFORM_CONFIG");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_ESP_IDF");

    let is_espidf = env::var("CARGO_FEATURE_ESP_IDF").is_ok()
        && env::var("CARGO_CFG_TARGET_OS").unwrap() == "espidf";

    if is_espidf
        && (env::var("WAMR_BUILD_PLATFORM").is_ok()
            || env::var("WAMR_SHARED_PLATFORM_CONFIG").is_ok())
    {
        panic!("ESP-IDF build cannot use custom platform build (WAMR_BUILD_PLATFORM) or shared platform config (WAMR_SHARED_PLATFORM_CONFIG)");
    }
    // because the ESP-IDF build procedure differs from the regular one (build internally by esp-idf-sys),
    else {
        let enable_custom_section = if cfg!(feature = "custom-section") {
            "1"
        } else {
            "0"
        };
        let enable_dump_call_stack = if cfg!(feature = "dump-call-stack") {
            "1"
        } else {
            "0"
        };
        let enable_llvm_jit = if cfg!(feature = "llvmjit") { "1" } else { "0" };
        let enable_multi_module = if cfg!(feature = "multi-module") {
            "1"
        } else {
            "0"
        };
        let enable_name_section = if cfg!(feature = "name-section") {
            "1"
        } else {
            "0"
        };

        let mut cfg = Config::new(&wamr_root);
        let mut cfg = cfg
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
            // hw bound checker (workaround for runwasi)
            .define("WAMR_DISABLE_HW_BOUND_CHECK", "1")
            // wamr private features
            .define("WAMR_BUILD_MULTI_MODULE", enable_multi_module)
            // - for developer
            .define("WAMR_BUILD_DUMP_CALL_STACK", enable_dump_call_stack)
            .define("WAMR_BUILD_CUSTOM_NAME_SECTION", enable_name_section)
            .define("WAMR_BUILD_LOAD_CUSTOM_SECTION", enable_custom_section);

        if let Ok(platform_name) = env::var("WAMR_BUILD_PLATFORM") {
            cfg.define("WAMR_BUILD_PLATFORM", &platform_name);
        }

        if let Ok(platform_config) = env::var("WAMR_SHARED_PLATFORM_CONFIG") {
            cfg.define("SHARED_PLATFORM_CONFIG", &platform_config);
            println!("cargo:rerun-if-changed={}", platform_config);
        }

        // support STDIN/STDOUT/STDERR redirect.
        cfg = match env::var("WAMR_BH_VPRINTF") {
            Ok(bh_vprintf) => match bh_vprintf.len() {
                0 => cfg,
                _ => cfg.define("WAMR_BH_VPRINTF", bh_vprintf),
            },
            Err(_) => cfg,
        };

        if enable_llvm_jit == "1" {
            let llvm_lib_path = env::var("LLVM_LIB_CFG_PATH").unwrap();
            cfg = cfg.define("LLVM_DIR", llvm_lib_path);
        }

        // set target and finish configuration
        let dst = cfg.build_target("iwasm_static").build();

        println!("cargo:rustc-link-search=native={}/build", dst.display());
        println!("cargo:rustc-link-lib=static=vmlib");

        //TODO: support macos?
        if enable_llvm_jit == "1" {
            println!("cargo:rustc-link-lib=dylib=dl");
            println!("cargo:rustc-link-lib=dylib=m");
            println!("cargo:rustc-link-lib=dylib=rt");
            println!("cargo:rustc-link-lib=dylib=stdc++");
            println!("cargo:rustc-link-lib=dylib=z");

            let llvm_dir = PathBuf::from(env::var("LLVM_LIB_CFG_PATH").unwrap());
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
