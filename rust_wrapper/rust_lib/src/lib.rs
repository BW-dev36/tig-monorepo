extern crate rand;

use rand::{rngs::StdRng, Rng, SeedableRng};
use std::ptr;

// Fonction pour créer un nouveau RNG avec une seed donnée
#[no_mangle]
pub extern "C" fn create_rng(seed: u64) -> *mut StdRng {
    let rng = StdRng::seed_from_u64(seed);
    Box::into_raw(Box::new(rng))
}

// Fonction pour générer un booléen aléatoire avec une probabilité donnée
#[no_mangle]
pub extern "C" fn gen_bool(rng_ptr: *mut StdRng, probability: f64) -> bool {
    let rng = unsafe {
        assert!(!rng_ptr.is_null());
        &mut *rng_ptr
    };
    rng.gen_bool(probability)

}

// Fonction pour générer un nombre aléatoire dans une plage donnée
#[no_mangle]
pub extern "C" fn gen_range(rng_ptr: *mut StdRng, min: u32, max: u32) -> u32 {
    let rng = unsafe {
        assert!(!rng_ptr.is_null());
        &mut *rng_ptr
    };
    rng.gen_range(min..(max as u32))
}

// Fonction pour détruire le RNG et libérer la mémoire
#[no_mangle]
pub extern "C" fn destroy_rng(rng_ptr: *mut StdRng) {
    if !rng_ptr.is_null() {
        unsafe {
            Box::from_raw(rng_ptr);
        }
    }
}
