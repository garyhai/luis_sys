use env_logger;
use futures::{Future, Stream};
use log::{error, info};
use luis_sys::{builder::Builder, events::Flags, Result};
use std::env;
use tokio;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "debug");
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
    recognize_once(&factory).map_err(|e| dbg!(e)).unwrap();
    recognize_stream(&factory).map_err(|e| dbg!(e)).unwrap();
    recognize_json(&factory).map_err(|e| dbg!(e)).unwrap();
    recognize_text(&factory).map_err(|e| dbg!(e)).unwrap();
    info!("Stop ASR test...");
}

fn recognize_once(factory: &Builder) -> Result {
    info!("Synchronous ASR ");
    let recognizer = factory.build()?;
    let result = recognizer.recognize()?;
    info!("done: {}", result);
    Ok(())
}

fn recognize_stream(factory: &Builder) -> Result {
    info!("Asynchronous ASR, streaming Event object");
    let mut reco = factory.build()?;
    let promise = reco.start()?.for_each(|msg| {
        info!("result: {:?}", msg.into_result());
        Ok(())
    });
    tokio::run(promise);
    Ok(())
}

fn recognize_json(factory: &Builder) -> Result {
    info!("Asynchronous ASR, get json results");
    let mut reco = factory.build()?;
    let promise = reco
        .start()?
        .filter(Flags::Recognized)
        .json()
        .for_each(|msg| {
            info!("result: {}", msg);
            Ok(())
        })
        .map_err(|err| error!("{}", err));

    tokio::run(promise);
    Ok(())
}

fn recognize_text(factory: &Builder) -> Result {
    info!("Asynchronous ASR, get text only results.");
    let mut reco = factory.build()?;
    let promise = reco.start()?;
    let promise = promise
        .text()
        .for_each(move |msg| {
            info!("result: {}", msg);
            Ok(())
        })
        .then(move |_| reco.stop())
        .map_err(|err| error!("{}", err));
    tokio::run(promise);
    Ok(())
}
