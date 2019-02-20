use env_logger;
use log::info;
use luis_sys::{asr::builder::Builder, Result};
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "trace");
    env_logger::init();
    info!("Start ASR test...");
    recognize_once().map_err(|e| dbg!(e)).unwrap();
    info!("Stop ASR test...");
}

fn recognize_once() -> Result {
    let recognizer = Builder::new()
        .subscription_key("d5504c34dab74874930d3fe9f2925578")
        .region("eastasia")
        .language("zh-CN")
        .audio_file_path("examples/chinese_test.wav")
        .create_recognizer()?;
    let result = recognizer.recognize()?;
    info!("done: {}", result);
    Ok(())
}