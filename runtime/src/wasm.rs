use crate::RadiantRuntime;
use crate::{RectangleTool, Runtime};
use radiant_core::View;
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = RadiantAppController)]
pub struct RadiantAppController {
    runtime: Arc<RwLock<RadiantRuntime>>,
}

#[wasm_bindgen(js_class = RadiantAppController)]
impl RadiantAppController {
    #[wasm_bindgen(constructor)]
    pub async fn new(f: &js_sys::Function) -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Error).expect("Couldn't initialize logger");

        let mut runtime = RadiantRuntime::new().await;
        runtime
            .view
            .scene_mut()
            .tool_manager
            .register_tool(1u32, Box::new(RectangleTool::new()));

        let runtime = Arc::new(RwLock::new(runtime));

        radiant_winit::run_wasm(runtime.clone(), f.clone());

        Self { runtime }
    }

    #[wasm_bindgen(js_name = handleMessage)]
    pub fn handle_message(&mut self, message: JsValue) {
        if let Ok(message) = serde_wasm_bindgen::from_value(message.clone()) {
            if let Ok(mut runtime) = self.runtime.write() {
                runtime.handle_message(message);
            }
        } else {
            log::error!("Couldn't deserialize message {:?}", message);
        }
    }
}