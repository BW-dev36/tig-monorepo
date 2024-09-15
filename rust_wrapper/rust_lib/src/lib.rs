extern crate rand;

use rand::distributions::{Distribution, Uniform, uniform::{SampleUniform, UniformSampler}};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct RngArrayNative {
    rngs: [StdRng; 8],
    index: u32,
}

impl RngArrayNative {
    pub fn new(seeds: [u64; 8]) -> Self {
        let rngs = seeds.map(StdRng::seed_from_u64);
        RngArrayNative { rngs, index: 0 }
    }

    pub fn get_mut(&mut self) -> &mut StdRng {
        self.index = (&mut self.rngs[self.index as usize]).gen_range(0..8);
        &mut self.rngs[self.index as usize]
    }

    // Fonction générique pour f32 (float) et f64 (double)
    pub fn sample_uniform<T>(&mut self, low: T, high: T) -> T
    where
        T: PartialOrd + Copy + SampleUniform, // Contraintes sur les types
    {
        let uniform = Uniform::from(low..high);
        uniform.sample(self.get_mut())
    }
}

// Expose RngArrayNative to C via FFI
#[no_mangle]
pub extern "C" fn rng_array_native_new(seeds: *const u64) -> *mut RngArrayNative {
    // Safety: Assumes `seeds` is a valid pointer to an array of 8 u64 values
    let seeds_array: [u64; 8] = unsafe { *(seeds as *const [u64; 8]) };
    let instance = RngArrayNative::new(seeds_array);
    Box::into_raw(Box::new(instance))
}

#[no_mangle]
pub extern "C" fn rng_array_native_free(ptr: *mut RngArrayNative) {
    if !ptr.is_null() {
        // Convert the raw pointer back into a box and drop it
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn rng_array_native_sample_uniform64(
    ptr: *mut RngArrayNative,
    low: f64,
    high: f64,
) -> f64 {
    if ptr.is_null() {
        return 0.0;
    }
    let rng = unsafe { &mut *ptr };
    rng.sample_uniform(low, high)
}

#[no_mangle]
pub extern "C" fn rng_array_native_sample_uniform32(
    ptr: *mut RngArrayNative,
    low: f32,
    high: f32,
) -> f32 {
    if ptr.is_null() {
        return 0.0;
    }
    let rng = unsafe { &mut *ptr };
    rng.sample_uniform(low, high)
}

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
            let _ = Box::from_raw(rng_ptr);
        }
    }
}
