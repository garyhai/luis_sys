# luis_sys
Rust FFI bindings for Microsoft LUIS API.

**A rust style wrapper for Microsoft LUIS C/C++ SDK.**

## Usage

Add luis_sys to the dependencies section in your project's `Cargo.toml`, with

```toml
[dependencies]
luis_sys = "^0.3.8"
```

Note: The crate includes [Cognitive Services Speech SDK Linux Version](https://aka.ms/csspeech/linuxbinary) 1.3.1. Windows version is not tested.

## Example

Create entry main function with crates of luis_sys, logger and futures.

```rust
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
```

Construct a builder by subscription info with configurations. The audio input is a wav file in `example` folder.

```rust
    let mut factory = RecognizerConfig::from_subscription(
        "YourLanguageUnderstandingSubscriptionKey",
        "YourLanguageUnderstandingServiceRegion",
    )?;

    // Choose the events to subscribe.
    let flags = Flags::Recognition
        | Flags::SpeechDetection
        | Flags::Session
        | Flags::Connection
        | Flags::Canceled;

    // Add intents if you want a intent recognizer. They are phrases or intents names of a pre-trained language understanding model.
    let intents = vec![
        "否定".to_string(),
        "肯定".to_string(),
        "中秋快乐祝你们平安无事快乐健康的生活".to_string(),
        "祝福".to_string(),
    ];

    factory
        .set_flags(flags)
        .set_audio_file_path("examples/chinese_test.wav")
        .set_model_id("YourLanguageUnderstandingAppId")
        .set_intents(intents)
        .put_language("TheLanguageOfAudioInput")?;
        .put_detailed_result(true)?;

```

`factory.recognizer()` build a speech recognition only recognizer.
`factory.intent_recognizer()` build a speech intent recognizer.

Starts blocked intent recognition, and returns after a single utterance. The end of a single utterance is determined by listening for silence at the end or until a maximum of 15 seconds of audio is processed. 

```rust
fn recognize_once(factory: &RecognizerConfig) -> Result {
    info!("Synchronous ASR ");
    let recognizer = factory.recognizer()?;
    let result = recognizer.recognize()?;
    info!("done: {}", result);
    Ok(())
}

```

Asynchronous intent recognition in tokio runtime.

```rust
fn recognize_stream(factory: &RecognizerConfig) -> Result {
    info!("Asynchronous ASR, streaming Event object");
    let mut reco = factory.intent_recognizer()?;
    let promise = reco
        .start()?
        // Add event filter to choice events you care.
        .set_filter(Flags::Recognized | Flags::SpeechDetection)
        .for_each(|msg| {
            info!("result: {:?}", msg.into_result());
            Ok(())
        });
    tokio::run(promise);
    Ok(())
}

```

Translate and synthesis audio.

```rust
factory
    // Add one or many target languages to tranlate from speech.
    .add_target_language("en")?
    // Enable audio synthesis output.
    .put_translation_features("textToSpeech")?
    // Select voice name appropriate for the target language.
    .put_voice_name("Microsoft Server Speech Text to Speech Voice (en-US, JessaRUS)")?;

info!("Asynchronous translation and audio synthesis");
let mut reco = factory.translator()?;
let promise = reco
    .start()?
    .set_filter(Flags::Recognized | Flags::Synthesis)
    .for_each(|evt| {
        // Handle the translation or synthesis result.
        Ok(())
    })
    .map_err(|err| error!("{}", err));

tokio::run(promise);

```

`EventStream` returned by `Recognizer::start` is implemented `futures::Stream `for asynchronous operation. And it can be refined by `set_filter`, `resulting`, `json` and `text` to pump different format results. And you can do that and more by Future/Stream combinations.

## Versions

See the [change log](https://github.com/neunit/luis_sys/blob/master/CHANGELOG.md).

## Notice

- The crate is working in progress, carefully if apply in production.

- Only speech SDK of LUIS service has C/C++ version. So current version supports very few feature of LUIS while LUIS SDK is in fast evolution phase.
- Windows version SDK is not tested.
- Linux version SDK only support Ubuntu distribution currently.
- Please read the [prerequisites](https://docs.microsoft.com/azure/cognitive-services/speech-service/quickstart-cpp-linux) at first.
