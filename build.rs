// build.rs
use bindgen;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    match env::consts::OS {
        "linux" => linux(),
        "macos" => macos(),
        _ => (),
    };
}

fn linux() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy(
        "./SpeechSDK/linux/lib/x64/libMicrosoft.CognitiveServices.Speech.core.so",
        Path::new(&out_path).join(
            "libMicrosoft.CognitiveServices.Speech.core.so"
        )
    ).unwrap();

    println!("cargo:rustc-link-search=native={}", out_path.display());
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
        .clang_arg("-ISpeechSDK/linux/include/c_api/")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn macos() {
    println!("cargo:rustc-link-search=framework={}", "SpeechSDK/macos");
    println!("cargo:rustc-link-lib=framework=MicrosoftCognitiveServicesSpeech");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("c_api/wrapper.h")
        .clang_arg("-ISpeechSDK/macos/MicrosoftCognitiveServicesSpeech.framework/Headers")
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
