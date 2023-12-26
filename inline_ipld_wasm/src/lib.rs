use inline_ipld::store::memory::MemoryStore;
use wasm_bindgen::prelude::*;
use web_sys::console;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Your code goes here!
    console::log_1(&JsValue::from_str("Hello world!"));

    Ok(())
}

// #[wasm_bindgen]
// extern "C" {
//     fn alert(s: &str);
// }

// #[wasm_bindgen]
// #[derive(Copy, Clone, Debug)]
// pub struct JsMemStore {}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello!");
}

#[wasm_bindgen]
pub struct InlineIpld();

#[wasm_bindgen]
impl InlineIpld {
    pub fn add_one(&self, x: u32) -> u32 {
        x + 1
    }

    // pub fn new_memory_store(&self) -> MemoryStore {
    //     MemoryStore::new()
    // }
}
