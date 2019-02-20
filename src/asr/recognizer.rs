use crate::speech_api::*;
use crate::{
    audio::AudioInput, hr, DeriveSpxHandle, Handle,
    Result, SpxError, SpxHandle,
};
use super::builder::*;
use super::results::*;
use std::ptr::null_mut;

pub enum EventType {
    Session,
    SpeechDetected,
    Recognizing,
    Recognized,
    Canceled,
}

DeriveSpxHandle!(
    Recognizer,
    recognizer_handle_release,
    recognizer_handle_is_valid
);

pub struct Recognizer {
    handle: SPXRECOHANDLE,
    config: RecognizerConfig,
    audio: AudioInput,
}

impl Recognizer {
    pub fn from_config(
        config: RecognizerConfig,
        audio: AudioInput,
    ) -> Result<Self> {
        let mut handle = null_mut();
        unsafe {
            hr(recognizer_create_speech_recognizer_from_config(
                &mut handle,
                config.handle(),
                audio.handle(),
            ))?;
            Ok(Recognizer {
                handle,
                config,
                audio,
            })
        }
    }

    pub fn from_subscription(subscription: &str, region: &str) -> Result<Self> {
        let config = RecognizerConfig::from_subscription(subscription, region)?;
        let audio = AudioInput::new();
        Recognizer::from_config(config, audio)
    }

    pub fn config(&self) -> &RecognizerConfig {
        &self.config
    }

    pub fn audio(&self) -> &AudioInput {
        &self.audio
    }

    pub fn recognize(&self) -> Result<String> {
        let mut hres = null_mut();
        unsafe {
            hr(recognizer_recognize_once(self.handle, &mut hres))?;
        }
        let mut rr = RecognitionResult::new(hres);
        if rr.reason()? == Result_Reason_ResultReason_RecognizedSpeech {
            Ok(String::from(rr.text()?))
        } else {
            Err(SpxError::Unknown)
        }
    }

    // pub fn subscribe(&mut self, events: &[EventType]) -> impl Stream<Item = Event, Error = SpxError> {
    //     let (p, c) = mpsc::unbounded::<Event>();
    //     for ev in events {
    //         match ev {
    //             EventType::Recognized => {

    //             }
    //         }
    //     }

    // }
}

