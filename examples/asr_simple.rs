//! Examples for luis_sys usage.

use env_logger;
use futures::{Future, Stream};
use log::{error, info};

use hound::WavReader;
use luis_sys::{
    builder::RecognizerConfig, events::Flags, AsrResult, CancellationResult,
    Recognizer, Result, SpeechResult,
};
use std::{env, io::Read};
use tokio;

const WAV_FILE: &str = "examples/chinese_test.wav";

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "debug,luis_sys=debug");
    env_logger::init();
    info!("Start ASR test...");
    recognize_test().unwrap();
    info!("Stop ASR test...");
}

fn recognize_test() -> Result {
    let wave = WavReader::open(WAV_FILE)?;
    let spec = wave.spec();
    let fmt_audio = (
        spec.sample_rate,
        spec.bits_per_sample as u8,
        spec.channels as u8,
    );

    let flags = Flags::Recognition
        | Flags::Synthesis
        | Flags::SpeechDetection
        | Flags::Session
        | Flags::Connection
        | Flags::Canceled;

    let mut factory = RecognizerConfig::from_subscription(
        "YourSubscriptionKey",
        "Region",
    )?;

    factory
        .set_flags(flags)
        // .set_audio_file_path(WAV_FILE)
        .set_audio_spec(fmt_audio)
        .set_pull_mode(true)
        // .set_intents(intents)
        .put_language("zh-CN")?;
    // .put_endpoint("endpoint ID of customized model")?;
    // .add_target_language("en")?
    // .put_translation_features("textToSpeech")?
    // .put_voice_name(
    // "Microsoft Server Speech Text to Speech Voice (en-US, JessaRUS)",
    // )?;
    // .put_detailed_result(true)?;
    let mut reader = wave.into_inner();

    // recognize_once(&factory).map_err(|e| dbg!(e))?;
    recognize_stream(&factory, &mut reader).map_err(|e| dbg!(e))?;
    // recognize_json(&factory).map_err(|e| dbg!(e))?;
    // recognize_text(&factory).map_err(|e| dbg!(e))?;
    // translate(&factory).map_err(|e| dbg!(e))?;
    Ok(())
}

#[allow(dead_code)]
fn recognize_once(factory: &RecognizerConfig) -> Result {
    info!("Synchronous ASR ");
    let recognizer = factory.recognizer()?;
    let rr = recognizer.recognize()?;
    let reason = rr.reason();
    if reason.contains(Flags::Recognized) {
        info!("Recognized: {:?}", rr.text());
    } else if reason.contains(Flags::Canceled) {
        error!("Error: {:?}", rr.error_details());
    } else {
        error!("unhandled reason {:?}", reason);
    }
    info!("done");
    Ok(())
}

#[allow(dead_code)]
fn recognize_stream<T: Read>(
    factory: &RecognizerConfig,
    reader: &mut T,
) -> Result {
    info!("Asynchronous ASR, streaming Event object");
    let mut reco = factory.recognizer()?;
    let promise = reco
        .start()?
        .set_filter(Flags::Recognized | Flags::SpeechDetection)
        .for_each(|msg| {
            info!("result: {:?}", msg.into_result());
            Ok(())
        });
    streaming(&mut reco, reader, 320)?;
    tokio::run(promise);
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}

#[allow(dead_code)]
fn recognize_json(factory: &RecognizerConfig) -> Result {
    info!("Asynchronous ASR, get json results");
    let mut reco = factory.recognizer()?;
    let promise = reco
        .start()?
        // .set_filter(Flags::Recognized | Flags::SpeechDetection)
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
    let mut reco = factory.intent_recognizer()?;
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

#[allow(dead_code)]
fn translate(factory: &RecognizerConfig) -> Result {
    info!("Asynchronous translation and audio synthesis, get json results");
    let mut reco = factory.translator()?;
    let promise = reco
        .start()?
        // .set_filter(Flags::Recognized | Flags::Synthesis)
        // .json()
        .for_each(|msg| {
            info!("result: {:?}", msg.into_result());
            Ok(())
        })
        .map_err(|err| error!("{:?}", err));

    tokio::run(promise);
    Ok(())
}

fn streaming<T: Read>(
    reco: &mut Recognizer,
    reader: &mut T,
    block_size: usize,
) -> Result {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    if block_size == 0 {
        reco.write_stream(&mut buffer)?;
    }

    for block in buffer.chunks_mut(block_size) {
        reco.write_stream(block)?;
    }
    reco.close_stream()?;
    Ok(())
}
