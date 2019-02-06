// use futures::future::IntoFuture;
// use futures::future::Future;
// use env_logger;
// use futures::Stream;
// use log::{debug, info, error};
// use microsoft_speech::{
//     audio::AudioConfig,
//     recognizer::{events::RecognitionResultEvent, RecognitionResult, SpeechRecognizer},
//     PropertyId, SpeechConfig, SpxError,
// };
// use std::{env, time::Duration, thread::sleep};

// use tokio::prelude::*;

// fn main() {
//     env::set_var("RUST_BACKTRACE", "1");
//     env::set_var("RUST_LOG", "trace");
//     env_logger::init();
//     info!("Start ASR test...");

//     let log_err = |e: SpxError| {
//         error!("Error: {:?}", e);
//         e
//     };

//     let mut sc =
//         SpeechConfig::from_subscription("c5e3fe2700ae4a9592328976e1a33017", "eastasia").unwrap();
//     sc.set(PropertyId::SpeechServiceConnectionRecoLanguage, "zh-CN")
//         .unwrap();
//     sc.set(PropertyId::SpeechServiceResponseRequestDetailedResultTrueFalse, "true").unwrap();

//     debug!("{}", env::current_dir().unwrap().display());

//     let ac = AudioConfig::from_wav_file_input("examples/chinese_test.wav").unwrap();
//     let mut recognizer = SpeechRecognizer::from_config(sc, Some(ac)).map_err(log_err).unwrap();
//     let result = recognizer.recognize_once_async().map_err(log_err).unwrap();

//     let mut r = tokio::runtime::Runtime::new().unwrap();
//     print_event(r.block_on(result).map_err(log_err).unwrap());

//     info!("done");
// }

// // fn print_event(e: RecognitionResultEvent<RecognitionResult>) {
// fn print_event(r: RecognitionResult) {
//     debug!("event fired");
//     // let r = e.result().unwrap();
//     debug!(
//         "session: , id: {}, reason: {:?}, e-offset: , r-offset: {}, duration: {:?}, text: {}",
//         // e.session_id().unwrap(),
//         r.id().unwrap(),
//         r.reason().unwrap(),
//         // e.offset().unwrap(),
//         r.offset().unwrap(),
//         r.duration().unwrap(),
//         r.text().unwrap()
//     );
// }
