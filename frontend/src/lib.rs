mod api;
mod components;
mod constants;
mod error;
mod events;
mod performance;
mod rendering;
mod state;
mod styles;

use crate::components::App;
use wasm_bindgen::prelude::*;

# [wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
