use env_logger;
use futures::{Future, Stream};
use log::{error, info};
use luis_sys::{builder::RecognizerConfig, events::Flags, Result};
use std::env;
use tokio;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    info!("Start ASR test...");
    recognize_test().unwrap();
    info!("Stop ASR test...");
}

fn recognize_test() -> Result {
    let flags = Flags::Recognition
        | Flags::Session
        | Flags::Connection
        | Flags::SpeechDetection
        | Flags::Canceled;
    let mut factory = RecognizerConfig::from_subscription(
        "YourLanguageUnderstandingSubscriptionKey",
        "YourLanguageUnderstandingServiceRegion",
    )?;

    // let intents = vec![
    //     "否定".to_string(),
    //     "肯定".to_string(),
    //     "祝福".to_string(),
    // ];
    factory
        .set_flags(flags)
        .set_audio_file_path("examples/chinese_test.wav")
        .set_model_id("YourLanguageUnderstandingAppId")
        // .set_intents(intents)
        .put_language("zh-CN")?
        .put_detailed_result(true)?;

    // recognize_once(&factory).map_err(|e| dbg!(e))?;
    // recognize_stream(&factory).map_err(|e| dbg!(e))?;
    recognize_json(&factory).map_err(|e| dbg!(e))?;
    // recognize_text(&factory).map_err(|e| dbg!(e))?;
    Ok(())
}

#[allow(dead_code)]
fn recognize_once(factory: &RecognizerConfig) -> Result {
    info!("Synchronous ASR ");
    let recognizer = factory.recognizer()?;
    let result = recognizer.recognize()?;
    info!("done: {}", result);
    Ok(())
}

#[allow(dead_code)]
fn recognize_stream(factory: &RecognizerConfig) -> Result {
    info!("Asynchronous ASR, streaming Event object");
    let mut reco = factory.intent_recognizer()?;
    // let mut reco = factory.recognizer()?;
    let promise = reco.start()?.for_each(|msg| {
        info!("result: {:?}", msg.into_result());
        Ok(())
    });
    tokio::run(promise);
    Ok(())
}

#[allow(dead_code)]
fn recognize_json(factory: &RecognizerConfig) -> Result {
    info!("Asynchronous ASR, get json results");
    let mut reco = factory.intent_recognizer()?;
    let promise = reco
        .start()?
        .filter(Flags::Recognized | Flags::SpeechDetection)
        .json()
        .for_each(|msg| {
            info!("result: {}", msg);
            Ok(())
        })
        .map_err(|err| error!("{}", err));

    tokio::run(promise);
    Ok(())
}

#[allow(dead_code)]
fn recognize_text(factory: &RecognizerConfig) -> Result {
    info!("Asynchronous ASR, get text only results.");
    let mut reco = factory.recognizer()?;
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
