use env_logger;
use futures::Stream;
use log::{error, info};
use luis_sys::{builder::Builder, events::Flags, Result};
use std::env;
use tokio;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "trace");
    env_logger::init();

    let flags = Flags::Recognization
        | Flags::Session
        | Flags::Connection
        | Flags::SpeechDetection
        | Flags::Canceled;
    let factory = Builder::new()
        .subscription_key("d5504c34dab74874930d3fe9f2925578")
        .region("eastasia")
        .language("zh-CN")
        .audio_file_path("examples/chinese_test.wav")
        .flags(flags);
    info!("Start ASR test...");
    // recognize_once(factory).map_err(|e| dbg!(e)).unwrap();
    recognize_continue(factory).map_err(|e| dbg!(e)).unwrap();
    info!("Stop ASR test...");
}

fn recognize_once(factory: Builder) -> Result {
    let recognizer = factory.build()?;
    let result = recognizer.recognize()?;
    info!("done: {}", result);
    Ok(())
}

fn recognize_continue(factory: Builder) -> Result {
    let mut reco = factory.build()?;
    reco.start()?;
    let promise = reco
        .map_err(|err| {
            error!("Something wrong: {}", err);
        })
        .for_each(|msg| {
            info!("result: {:?}", msg);
            Ok(())
        });
    tokio::run(promise);
    Ok(())
}
