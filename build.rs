// build.rs
use bindgen;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    match env::consts::OS {
        "linux" => linux(),
        "macos" => macos(),
        _ => (),
    };
}

fn downlaod_sdk(sdk: &'static str) {
    if Path::new("SpeechSDK").join(sdk).exists() {
        return;
    }
    Command::new("make")
            .args(&[sdk])
            .status()
            .expect("failed to download Speech SDK!");
}

fn linux() {
    downlaod_sdk("linux_sdk");
    println!("cargo:rustc-link-search=native={}", "SpeechSDK/linux_sdk/lib/x64/");
    println!(
        "cargo:rustc-link-lib=dylib=Microsoft.CognitiveServices.Speech.core"
    );

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("c_api/wrapper.h")
        .clang_arg("-ISpeechSDK/linux_sdk/include/c_api/")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn macos() {
    downlaod_sdk("macos_sdk");
    println!("cargo:rustc-link-search=framework={}", "SpeechSDK/macos_sdk");
    println!("cargo:rustc-link-lib=framework=MicrosoftCognitiveServicesSpeech");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("c_api/wrapper.h")
        .clang_arg("-ISpeechSDK/macos_sdk/MicrosoftCognitiveServicesSpeech.framework/Headers")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
