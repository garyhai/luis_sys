// build.rs
use bindgen;
use std::{env, fs, path::PathBuf, process::Command};

const LINUX_SDK_URL: &str = "https://aka.ms/csspeech/linuxbinary";
const MACOS_SDK_URL: &str = "https://aka.ms/csspeech/macosbinary";

fn main() {
    let renew = env::var("RENEW_SDK").map(|v| v == "1").unwrap_or(false);
    match env::consts::OS {
        "linux" => linux(renew),
        "macos" => macos(renew),
        _ => (),
    };
}

fn download_file(url: &str, dst: &str) {
    Command::new("curl")
        .args(&["-SL", url, "-o", dst])
        .status()
        .expect("failed to download Speech SDK!");
}

fn linux(mut renew: bool) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sdk_path = out_path.join("SpeechSDK").join("linux");
    if !sdk_path.exists() {
        renew = true;
        fs::create_dir_all(&sdk_path).unwrap();
    }

    if renew {
        let dw_file = out_path.join("linux.sdk");
        let sdk_file = dw_file.to_str().unwrap();
        download_file(LINUX_SDK_URL, sdk_file);
        let args = [
            "--strip",
            "1",
            "-xzf",
            sdk_file,
            "-C",
            sdk_path.to_str().unwrap(),
        ];
        Command::new("tar").args(&args).status().unwrap();
    }

    let lib_path = sdk_path.join("lib").join("x64");
    println!("cargo:rustc-link-search=native={}", lib_path.display());
    println!(
        "cargo:rustc-link-lib=dylib=Microsoft.CognitiveServices.Speech.core"
    );

    let mut inc_arg = String::from("-I");
    inc_arg.push_str(sdk_path.join("include").join("c_api").to_str().unwrap());
    let bindings = bindgen::Builder::default()
        .header("c_api/wrapper.h")
        .clang_arg(inc_arg.as_str())
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn macos(mut renew: bool) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sdk_path = out_path.join("SpeechSDK").join("macos");
    if !sdk_path.exists() {
        renew = true;
        fs::create_dir_all(&sdk_path).unwrap();
    }

    if renew {
        let dw_file = out_path.join("macos.sdk");
        let sdk_file = dw_file.to_str().unwrap();
        download_file(MACOS_SDK_URL, sdk_file);
        let args = ["-q", sdk_file, "-d", sdk_path.to_str().unwrap()];
        Command::new("unzip").args(&args).status().unwrap();
    }

    println!("cargo:rustc-link-search=framework={}", sdk_path.display());
    println!("cargo:rustc-link-lib=framework=MicrosoftCognitiveServicesSpeech");

    let mut inc_arg = String::from("-I");
    let inc_path = sdk_path
        .join("MicrosoftCognitiveServicesSpeech.framework")
        .join("Headers");
    inc_arg.push_str(inc_path.to_str().unwrap());
    let bindings = bindgen::Builder::default()
        .header("c_api/wrapper.h")
        .clang_arg(inc_arg.as_str())
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
