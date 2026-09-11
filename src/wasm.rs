//! WASM bindings for vers-like-specifier
//!
//! This module provides WebAssembly bindings for the vers-like-specifier library, allowing it to be used in web applications.

use wasm_bindgen::prelude::wasm_bindgen;

/// Initialize panic hook for better error messages in the browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
