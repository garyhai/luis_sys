use env_logger;
use log::{info, error};
use luis_sys::{builder::Builder, Result, recognizer::Recognizer, events::Flags};
use std::env;
use futures::{Future, Stream};
use tokio;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "trace");
    env_logger::init();
    info!("Start ASR test...");
    let factory = Builder::new()
        .subscription_key("d5504c34dab74874930d3fe9f2925578")
        .region("eastasia")
        .language("zh-CN")
        .audio_file_path("examples/chinese_test.wav");
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

fn recognize_continue(mut factory: Builder) -> Result {
    let mut reco = factory.build()?;
    reco.start(Flags::empty())?;
    let promise = reco.map_err(|err| {
        error!("Something wrong: {}", err);
    }).for_each(|evt| {
        match evt.into_result() {
            Ok(res) => info!("result: {:?}", res),
            Err(err) => error!("error: {:?}", err),
        }
        Ok(())
    }).wait();
    // tokio::run(promise);
    Ok(())
}