mod benchmarker;
mod future_utils;

#[cfg(feature = "browser")]
mod exports {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub async fn state() -> JsValue {
        let state = benchmarker::state().lock().await.clone();
        let value = serde_json::to_value(&state).unwrap();
        serde_wasm_bindgen::to_value(&value).unwrap()
    }

    #[wasm_bindgen]
    pub async fn start(num_workers: u32) {
        benchmarker::start(num_workers).await;
    }

    #[wasm_bindgen]
    pub async fn stop() {
        benchmarker::stop().await;
    }

    #[wasm_bindgen]
    pub async fn select_algorithm(challenge_id: String, algorithm_id: String) {
        benchmarker::select_algorithm(challenge_id, algorithm_id).await;
    }

    #[wasm_bindgen]
    pub async fn setup(api_url: String, api_key: String, player_id: String) {
        benchmarker::setup(api_url, api_key, player_id.to_string()).await;
    }
}
