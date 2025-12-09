//! Integration tests for WASM build
//!
//! These tests verify that the WASM module can be built and basic functionality works.
//! Note: These tests require wasm-pack to be installed.

use std::process::Command;
use std::path::Path;

#[test]
#[ignore] // Ignore by default since it requires wasm-pack
fn test_wasm_build() {
    // Check if wasm-pack is available
    let wasm_pack_check = Command::new("wasm-pack")
        .arg("--version")
        .output();

    if wasm_pack_check.is_err() {
        eprintln!("Skipping WASM test: wasm-pack not found. Install with: cargo install wasm-pack");
        return;
    }

    // Try to build WASM module
    let build_result = Command::new("wasm-pack")
        .args(&["build", "--target", "web", "--out-dir", "examples/web-editor/pkg"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output();

    match build_result {
        Ok(output) => {
            if output.status.success() {
                println!("WASM build successful!");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("WASM build failed:\n{}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run wasm-pack: {}", e);
        }
    }

    // Verify output files exist
    let pkg_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/web-editor/pkg");
    assert!(pkg_dir.exists(), "WASM package directory should exist");
    
    let wasm_file = pkg_dir.join("figurehead_bg.wasm");
    assert!(wasm_file.exists(), "WASM binary should exist");
    
    let js_file = pkg_dir.join("figurehead.js");
    assert!(js_file.exists(), "JavaScript bindings should exist");
}

#[test]
#[ignore]
fn test_wasm_module_structure() {
    // This test verifies that the WASM module exports the expected functions
    // In a real scenario, you might use wasm-bindgen-test or a WASM runtime
    
    // For now, we just verify the source code compiles for WASM target
    let check_result = Command::new("cargo")
        .args(&["check", "--target", "wasm32-unknown-unknown"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output();

    match check_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("WASM target check failed:\n{}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to check WASM target: {}", e);
        }
    }
}
