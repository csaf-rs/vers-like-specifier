//! WASM bindings for the vls (VERS-like specifier) parser library
//!
//! This module provides WebAssembly bindings for the VERS-like specifier library, allowing it to be used in web applications.

use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in the browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
