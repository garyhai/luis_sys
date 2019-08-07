//! Examples for luis_sys usage.

use env_logger;
use futures::Stream;
use log::{error, info};

use luis_sys::{
    builder::RecognizerConfig, events::*, CancellationResult, Result,
};
use std::env;
use tokio;

const TEXT: &str = "你好！";
const TEXT2: &str = "把你的手放在滚热的炉子上一分钟，感觉起来像一小时。坐在一个漂亮姑娘身边整整一小时，感觉起来像一分钟。这就是相对论。";

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "debug,luis_sys=trace");
    env_logger::init();
    info!("Start TTS test...");
    synthesis_test().unwrap();
    info!("Stop TTS test...");
}

fn synthesis_test() -> Result {
    let flags = Flags::SynthesisEvent | Flags::Synthesis | Flags::Canceled;

    let mut factory =
        RecognizerConfig::from_subscription("YourSubscriptionKey", "Region")?;

    factory
        .set_flags(flags)
        .set_audio_spec((8000, 16, 1))
        .put_synth_language("zh-CN")?;

    // synthesis_once(&factory).map_err(|e| dbg!(e))?;
    synthesis_stream(&factory).map_err(|e| dbg!(e))?;
    Ok(())
}

#[allow(dead_code)]
fn synthesis_once(factory: &RecognizerConfig) -> Result {
    info!("Synchronous TTS ");
    let mut synth = factory.synthesizer()?;
    let rr = synth.synthesis_once(TEXT)?;
    let reason = rr.reason();
    if reason.contains(Flags::Synthesized) {
        info!("Synthesized: {:?} bytes", rr.audio_data_length());
    } else if reason.contains(Flags::Canceled) {
        error!("Error: {:?}", rr.error_details());
    } else {
        error!("unhandled reason {:?}", reason);
    }
    info!("done");
    Ok(())
}

#[allow(dead_code)]
fn synthesis_stream(factory: &RecognizerConfig) -> Result {
    info!("Asynchronous TTS, streaming Event object");
    let mut synth = factory.synthesizer()?;
    let promise = synth
        .start()?
        // .set_filter(Flags::Recognition | Flags::SpeechDetection)
        .for_each(|msg| {
            // let res = SynthEventResult::from_event(msg).unwrap();
            // let length = res.audio_data_length().unwrap();
            // info!("result: {:?}: {:?}", res.reason(), length);
            let res = msg.into_synth_result().unwrap();
            info!(
                "result: {:?} with audio data {}",
                res.flag, res.audio_length
            );
            Ok(())
        });
    synth.start_synthesize(TEXT)?;
    synth.start_synthesize(TEXT2)?;
    tokio::run(promise);
    Ok(())
}
