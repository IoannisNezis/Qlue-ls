#[cfg(not(target_arch = "wasm32"))]
pub(super) fn get_timestamp() -> u128 {
    use std::time::Instant;
    Instant::now().elapsed().as_millis()
}

#[cfg(target_arch = "wasm32")]
pub(super) fn get_timestamp() -> u128 {
    use wasm_bindgen::JsCast;
    use web_sys::WorkerGlobalScope;
    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();
    worker_global
        .performance()
        .expect("performance should be available")
        .now() as u128
}
