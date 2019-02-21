use super::{builder::RecognizerConfig, results::RecognitionResult};
use crate::{
    audio::AudioInput,
    get_cf_string, hr,
    speech_api::{
        recognizer_async_handle_is_valid, recognizer_async_handle_release,
        recognizer_create_speech_recognizer_from_config, recognizer_disable,
        recognizer_enable, recognizer_event_handle_is_valid,
        recognizer_event_handle_release, recognizer_handle_is_valid,
        recognizer_handle_release, recognizer_recognize_once,
        recognizer_session_event_get_session_id, session_handle_is_valid,
        session_handle_release, Result_Reason_ResultReason_RecognizedSpeech,
        SPXEVENTHANDLE, SPXRECOHANDLE,
    },
    DeriveSpxHandle, Handle, Result, SmartHandle, SpxError, SpxHandle,
};
use std::{os::raw::c_void, ptr::null_mut};

pub enum EventType {
    Session,
    SpeechDetected,
    Recognizing,
    Recognized,
    Connection,
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
        let audio = AudioInput::default();
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

    pub fn pause(&self) -> Result {
        hr(unsafe { recognizer_disable(self.handle) })
    }

    pub fn resume(&self) -> Result {
        hr(unsafe { recognizer_enable(self.handle) })
    }

    // pub fn start(&mut self) -> Result {}

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

SmartHandle!(
    RecognizerEvent,
    recognizer_event_handle_release,
    recognizer_event_handle_is_valid
);

impl RecognizerEvent {
    pub fn session_id(&self) -> Result<String> {
        get_cf_string(recognizer_session_event_get_session_id, self.handle, 40)
    }
}

SmartHandle!(
    RecognizerAsync,
    recognizer_async_handle_release,
    recognizer_async_handle_is_valid
);

SmartHandle!(
    RecognizerSession,
    session_handle_release,
    session_handle_is_valid
);

unsafe extern "C" fn on_session_started(
    hreco: SPXRECOHANDLE,
    hevent: SPXEVENTHANDLE,
    context: *mut c_void,
) {
    let evt = RecognizerEvent::from(hevent as Handle);
    if context.is_null() {
        return;
    };
    let sid = evt.session_id().expect("unexpected error");
    let recognizer: &mut Recognizer =
        unsafe { &mut *(context as *mut Recognizer) };
}
