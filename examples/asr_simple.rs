use env_logger;
use log::{debug, info, error};
use luis_sys::{
    Result,
    audio::AudioConfig,
    asr::{RecognizerConfig, Recognizer},
};
use std::env;

fn main() -> Result {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "trace");
    env_logger::init();
    info!("Start ASR test...");

    let sc =
        RecognizerConfig::from_subscription("c5e3fe2700ae4a9592328976e1a33017", "eastasia")?;
    sc.set_language("zh-CN")?;
    let ac = AudioConfig::from_wav_file_input("examples/chinese_test.wav")?;
    let recognizer = Recognizer::new(sc, ac)?;
    let result = recognizer.recognize()?;
    info!("done: {}", result);
    Ok(())
}
