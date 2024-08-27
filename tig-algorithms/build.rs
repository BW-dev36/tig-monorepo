use std::env;
use std::process::Command;
use std::path::PathBuf;

fn main() {
    let profile = env::var("PROFILE").unwrap();
    let output_dir = env::var("OUT_DIR").unwrap();
    let cuda_src_dir = "native"; // Mise à jour du chemin relatif

    // // Compile the Rust library
    // let status = Command::new("cargo")
    //     .current_dir("../rust_wrapper/rust_lib")
    //     .args(&["-Z", "unstable-options", "build", "--release", "--out-dir", format!("{}", env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| output_dir.clone())).as_str()])
    //     .status()
    //     .expect("Failed to build Rust library");

    // assert!(status.success(), "Cargo build failed");


    // Obtenir le répertoire de travail actuel
    let current_dir = env::current_dir().expect("Failed to get current directory");

    // Construire le chemin absolu pour le répertoire source
    let src_dir = current_dir.join(cuda_src_dir);
    let src_dir_str = src_dir.to_str().expect("Failed to convert src_dir to string");

    // Vérifier que le répertoire source existe
    if !src_dir.exists() {
        panic!("The source directory {} does not exist.", src_dir_str);
    }

    // Définir les répertoires de build et de source
    let build_dir = PathBuf::from(&output_dir).join("build");

    // Créer le répertoire de build s'il n'existe pas
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    // Vérifier la présence du toolkit CUDA
    let _cuda_path = PathBuf::from("/usr/local/cuda");
    let enable_cuda = false; //cuda_path.exists();

    // Construire les arguments pour CMake
    let mut cmake_args = vec![
        src_dir_str.to_string(),
        "--fresh".to_string(),
        format!("-DCMAKE_BUILD_TYPE={}", profile),
        format!("-DCMAKE_LIBRARY_OUTPUT_DIRECTORY={}", build_dir.to_str().unwrap()),
        format!("-DRUST_LIB_DIR={}", env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| output_dir.clone())),
    ];

    // Ajouter les arguments spécifiques à CUDA si disponible
    if enable_cuda {
        cmake_args.push("-DENABLE_CUDA=ON".to_string());
    } else {
        cmake_args.push("-DENABLE_CUDA=OFF".to_string());
    }

    // Exécuter CMake
    let cmake_status = Command::new("cmake")
        .args(&cmake_args)
        .current_dir(&build_dir)
        .status()
        .expect("Failed to run CMake");

    assert!(cmake_status.success(), "CMake configuration failed");

    // Construire le projet avec make
    let make_status = Command::new("make")
        .current_dir(&build_dir)
        .status()
        .expect("Failed to build with make");

    assert!(make_status.success(), "Make build failed");



    println!("cargo:rustc-link-search=native={}", build_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=cpp_cuda");
    println!("cargo:rerun-if-changed={}/knapmaxxing.cu", src_dir_str);
    println!("cargo:rustc-link-search=native=/usr/lib/gcc/x86_64-linux-gnu/12/");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    // Lien vers la bibliothèque construite
    // println!("cargo:rustc-link-search=native={}", output_dir);
    // println!("cargo:rustc-link-lib=static=rust_lib");
   
}
