use super::{Job, Result};
use crate::future_utils::Mutex;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use tig_utils::get;

static CACHE: OnceCell<Mutex<HashMap<String, Vec<u8>>>> = OnceCell::new();

pub async fn execute(job: &Job) -> Result<Vec<u8>> {
    let mut cache = CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .await;

    if let Some(wasm_blob) = cache.get(&job.settings.algorithm_id) {
        Ok(wasm_blob.clone())
    } else {
        // Vérifiez si l'ID de l'algorithme est "c002_a036"
        let wasm = if job.settings.algorithm_id == "c002_a036" {
            println!("Loading wasm for vehicle routing...");
            // Charger le fichier .wasm localement
            let local_path = "/home/damien/tig-wynnh/tig-algorithms/wasm/vehicle_routing/clarke_wright_ultimate.wasm";
            let mut file = File::open(local_path)
                .map_err(|e| format!("Failed to open local wasm file {}: {:?}", local_path, e))?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read local wasm file {}: {:?}", local_path, e))?;
            buffer
        } else {
            // Télécharger le fichier .wasm
            get::<Vec<u8>>(&job.download_url, None)
                .await
                .map_err(|e| format!("Failed to download wasm from {}: {:?}", job.download_url, e))?
        };

        // Cache the wasm blob
        (*cache).insert(job.settings.algorithm_id.clone(), wasm.clone());
        Ok(wasm)
    }
}
