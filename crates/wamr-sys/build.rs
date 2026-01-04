/*
 * Copyright (C) 2023 Liquid Reply GmbH. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

use cmake::Config;
use std::{env, path::Path, path::PathBuf};

const LLVM_LIBRARIES: &[&str] = &[
    // keep alphabet order
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
];

fn check_is_espidf() -> bool {
    let is_espidf = env::var("CARGO_FEATURE_ESP_IDF").is_ok()
        && env::var("CARGO_CFG_TARGET_OS").unwrap() == "espidf";

    if is_espidf
        && (env::var("WAMR_BUILD_PLATFORM").is_ok()
            || env::var("WAMR_SHARED_PLATFORM_CONFIG").is_ok())
    {
        panic!(
            "ESP-IDF build cannot use custom platform build (WAMR_BUILD_PLATFORM) or shared platform config (WAMR_SHARED_PLATFORM_CONFIG)"
        );
    }

    is_espidf
}

macro_rules! wamr_build_enable_option {
    (not; $feature:expr) => {
        if cfg!(not(feature = $feature)) {
            "1"
        } else {
            "0"
        }
    };
    ($feature:expr) => {
        if cfg!(feature = $feature) { "1" } else { "0" }
    };
}

fn link_llvm_libraries(llvm_cfg_path: &str, enable_llvm_jit: &str) {
    if enable_llvm_jit == "0" {
        return;
    }

    let llvm_cfg_path = PathBuf::from(llvm_cfg_path);
    assert!(llvm_cfg_path.exists());

    let llvm_lib_path = llvm_cfg_path.join("../../../lib").canonicalize().unwrap();
    assert!(llvm_lib_path.exists());

    println!("cargo:rustc-link-lib=dylib=dl");
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=rt");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:libdir={}", llvm_lib_path.display());
    println!("cargo:rustc-link-search=native={}", llvm_lib_path.display());

    for &llvm_lib in LLVM_LIBRARIES {
        println!("cargo:rustc-link-lib=static={}", llvm_lib);
    }
}

fn setup_config(cmakelists_dir: &PathBuf) -> Config {
    let mut cfg = Config::new(cmakelists_dir);

    for (key, value) in [
        ("WAMR_BUILD_AOT", wamr_build_enable_option!("aot")),
        (
            "WAMR_BUILD_AOT_STACK_FRAME",
            wamr_build_enable_option!("aot-stack-frame"),
        ),
        (
            "WAMR_BUILD_INTERP",
            wamr_build_enable_option!("interpreter"),
        ),
        (
            "WAMR_BUILD_FAST_INTERP",
            wamr_build_enable_option!("fast-interpreter"),
        ),
        (
            "WAMR_BUILD_SHARED_MEMORY",
            wamr_build_enable_option!("shared-memory"),
        ),
        (
            "WAMR_BUILD_MINI_LOADER",
            wamr_build_enable_option!("mini-loader"),
        ),
        ("WAMR_BUILD_JIT", wamr_build_enable_option!("llvmjit")),
        ("WAMR_BUILD_FAST_JIT", wamr_build_enable_option!("fast-jit")),
        (
            "WAMR_BUILD_BULK_MEMORY",
            wamr_build_enable_option!("bulk-memory"),
        ),
        (
            "WAMR_BUILD_REF_TYPES",
            wamr_build_enable_option!("reference-types"),
        ),
        (
            "WAMR_BUILD_LIB_PTHREAD",
            wamr_build_enable_option!("pthread"),
        ),
        (
            "WAMR_BUILD_LIB_PTHREAD_SEMAPHORE",
            wamr_build_enable_option!("pthread-semaphore"),
        ),
        (
            "WAMR_BUILD_LIBC_WASI",
            wamr_build_enable_option!("libc-wasi"),
        ),
        (
            "WAMR_BUILD_LIBC_BUILTIN",
            wamr_build_enable_option!("libc-builtin"),
        ),
        (
            "WAMR_BUILD_LIBC_UVWASI",
            wamr_build_enable_option!("libc-uvwasi"),
        ),
        (
            "WAMR_DISABLE_HW_BOUND_CHECK",
            wamr_build_enable_option!(not; "hw-bound-check"),
        ),
        (
            "WAMR_BUILD_MULTI_MODULE",
            wamr_build_enable_option!("multi-module"),
        ),
        (
            "WAMR_BUILD_DUMP_CALL_STACK",
            wamr_build_enable_option!("dump-call-stack"),
        ),
        (
            "WAMR_BUILD_CUSTOM_NAME_SECTION",
            wamr_build_enable_option!("name-section"),
        ),
        (
            "WAMR_BUILD_LOAD_CUSTOM_SECTION",
            wamr_build_enable_option!("custom-section"),
        ),
        ("WAMR_BUILD_SIMD", wamr_build_enable_option!("simd")),
        ("WAMR_BUILD_LIB_SIMDE", wamr_build_enable_option!("simde")),
        ("WAMR_BUILD_GC", wamr_build_enable_option!("gc")),
        (
            "WAMR_BUILD_EXCE_HANDLING",
            wamr_build_enable_option!("legacy-exception-handling"),
        ),
        ("WAMR_BUILD_MEMORY64", wamr_build_enable_option!("memory64")),
        (
            "WAMR_BUILD_MULTI_MEMORY",
            wamr_build_enable_option!("multi-memory"),
        ),
        (
            "WAMR_BUILD_THREAD_MGR",
            wamr_build_enable_option!("threads-manager"),
        ),
        (
            "WAMR_BUILD_LIB_WASI_THREADS",
            wamr_build_enable_option!("wasi-threads"),
        ),
        (
            "WAMR_BUILD_TAIL_CALL",
            wamr_build_enable_option!("tail-call"),
        ),
        (
            "WAMR_BUILD_MEMORY_PROFILING",
            wamr_build_enable_option!("memory-profiling"),
        ),
        (
            "WAMR_BUILD_DEBUG_INTERP",
            wamr_build_enable_option!("debug-interpreter"),
        ),
        ("WAMR_BUILD_WASI_NN", wamr_build_enable_option!("wasi-nn")),
        (
            "WAMR_BUILD_SHRUNK_MEMORY",
            wamr_build_enable_option!("shrunk-memory"),
        ),
    ] {
        cfg.define(key, value);
    }

    cfg.define("WASM_API_EXTERN", "");
    cfg.define("WASM_RUNTIME_API_EXTERN", "");

    // always assume non-empty strings for these environment variables

    if let Ok(platform_name) = env::var("WAMR_BUILD_PLATFORM") {
        cfg.define("WAMR_BUILD_PLATFORM", &platform_name);
    }

    if let Ok(target_name) = env::var("WAMR_BUILD_TARGET") {
        cfg.define("WAMR_BUILD_TARGET", &target_name);
    }

    if let Ok(platform_config) = env::var("WAMR_SHARED_PLATFORM_CONFIG") {
        cfg.define("SHARED_PLATFORM_CONFIG", &platform_config);
    }

    if let Ok(llvm_cfg_path) = env::var("LLVM_LIB_CFG_PATH") {
        link_llvm_libraries(&llvm_cfg_path, wamr_build_enable_option!("llvmjit"));
        cfg.define("LLVM_DIR", &llvm_cfg_path);
    }

    // STDIN/STDOUT/STDERR redirect
    if let Ok(bh_vprintf) = env::var("WAMR_BH_VPRINTF") {
        cfg.define("WAMR_BH_VPRINTF", &bh_vprintf);
    }

    cfg
}

fn build_wamr_libraries(wamr_root: &PathBuf) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let vmbuild_path = out_dir.join("vmbuild");

    let dst = {
        let mut cfg = setup_config(wamr_root);
        println!("cargo:rustc-link-lib=static=iwasm");
        cfg.out_dir(vmbuild_path).build_target("vmlib").build()
    }
    .join("build");

    let dst = if cfg!(target_os = "windows") {
        globwalk::GlobWalkerBuilder::from_patterns(&dst, &["iwasm.lib"])
            .build()
            .expect("Failed to build glob walker")
            .filter_map(Result::ok)
            .next()
            .and_then(|entry| entry.path().parent().map(|p| p.to_path_buf()))
            .unwrap_or(dst)
    } else {
        dst
    };

    println!("cargo:rustc-link-search=native={}", dst.display());
}

fn build_wamrc(wamr_root: &Path) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wamrc_build_path = out_dir.join("wamrcbuild");

    let wamr_compiler_path = wamr_root.join("wamr-compiler");
    assert!(wamr_compiler_path.exists());

    Config::new(&wamr_compiler_path)
        .out_dir(wamrc_build_path)
        .define("WAMR_BUILD_WITH_CUSTOM_LLVM", "1")
        .define(
            "LLVM_DIR",
            env::var("LLVM_LIB_CFG_PATH")
                .expect("LLVM_LIB_CFG_PATH isn't specified in config.toml"),
        )
        .build();
}

fn generate_bindings(wamr_root: &Path) {
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

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_ESP_IDF");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    println!("cargo:rerun-if-env-changed=WAMR_BUILD_PLATFORM");
    println!("cargo:rerun-if-env-changed=WAMR_SHARED_PLATFORM_CONFIG");
    println!("cargo:rerun-if-env-changed=LLVM_LIB_CFG_PATH");
    println!("cargo:rerun-if-env-changed=WAMR_BH_VPRINTF");

    let wamr_root = env::current_dir().unwrap();
    let wamr_root = wamr_root.join("wasm-micro-runtime");
    assert!(wamr_root.exists());

    if !check_is_espidf() {
        // because the ESP-IDF build procedure differs from the regular one
        // (build internally by esp-idf-sys),

        std::thread::scope(|t| {
            let lib = t.spawn(|| {
                build_wamr_libraries(&wamr_root);
            });
            let wamrc = t.spawn(|| {
                // build_wamrc(&wamr_root);
            });
            lib.join().expect("lib thread panicked");
            wamrc.join().expect("wamrc thread panicked");
        });
    }

    generate_bindings(&wamr_root);
}
