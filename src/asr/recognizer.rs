use super::{
    builder::RecognizerConfig,
    results::{RecognitionResult, Results},
};
use crate::{
    audio::AudioInput,
    get_cf_string, hr,
    speech_api::{
        recognizer_async_handle_is_valid, recognizer_async_handle_release,
        recognizer_create_speech_recognizer_from_config, recognizer_disable,
        recognizer_enable, recognizer_event_handle_is_valid,
        recognizer_event_handle_release, recognizer_handle_is_valid,
        recognizer_handle_release, recognizer_recognition_event_get_offset,
        recognizer_recognize_once, recognizer_recognizing_set_callback,
        recognizer_session_event_get_session_id,
        recognizer_start_continuous_recognition_async,
        recognizer_start_continuous_recognition_async_wait_for,
        session_handle_is_valid, session_handle_release,
        Result_Reason_ResultReason_RecognizedSpeech, SPXEVENTHANDLE,
        SPXRECOHANDLE, UINT32_MAX, recognizer_recognized_set_callback,
    },
    DeriveSpxHandle, Handle, Result, SmartHandle, SpxError, SpxHandle,
};
use serde::Serialize;
use std::{
    os::raw::c_void,
    ptr::null_mut,
    sync::{Arc, Mutex, Weak},
};
use futures::sync::mpsc::{UnboundedSender};

macro_rules! DefCallback {
    ($name:ident, $flag:expr) => {
        unsafe extern "C" fn $name(
            hreco: SPXRECOHANDLE,
            hevent: SPXEVENTHANDLE,
            context: *mut c_void,
        ) {
            let evt = RecognizerEvent::new($flag, hevent);
            if context.is_null() {
                log::error!("Unknown context with NULL pointer.");
                return;
            }
            let context =
                unsafe { &mut *(context as *mut Weak<UnboundedSender<RecognizerEvent>>) };
            let context = Weak::clone(context);
            if let Some(arc) = context.upgrade() {
                let mut sender = Arc::make_mut(arc);
                if let Err(err) = sender.unbounded_send(evt) {
                    log::error!("failed to post event data by error: {}", err);
                    log::debug!("{:?}", err);
                }
            } else {
                log::error!("Recognizer instance is dropped!");
            }
        }
    };
}

bitflags! {
    #[derive(Default, Serialize)]
    pub struct EventFlags: u64 {
        const Connected = 0b0001;
        const Disconnected = 0b0010;
        const Connection = 0b0011;
        const SessionStarted = 0b0100;
        const SessionStopped = 0b1000;
        const Session = 0b1100;
        const SpeechStartDetected = 0b0001_0000;
        const SpeechEndDetected = 0b0010_0000;
        const SpeechDetection = 0b0011_0000;
        const Recognizing = 0b0100_0000;
        const Recognized = 0b1000_0000;
        const Recognization = 0b1100_0000;
        const Canceled = 0b0001_0000_0000;
    }
}

#[allow(non_camel_case_types)]
#[derive(Serialize)]
pub struct AsrEvent {
    tag: EventFlags,
    id: Option<String>,
    offset: Option<u64>,
    results: Option<Results>,
}

impl AsrEvent {
    pub fn new(tag: EventFlags, re: RecognizerEvent) -> Self {
        AsrEvent {
            tag,
            id: None,
            offset: None,
            results: None,
        }
    }
}

DeriveSpxHandle!(
    Recognizer,
    recognizer_handle_release,
    recognizer_handle_is_valid
);

pub struct Recognizer {
    handle: SPXRECOHANDLE,
    myself: Weak<Mutex<Self>>,
    hooks: EventFlags,
    subscribers: Vec<(EventFlags, Sender<AsrEvent>)>,
    config: RecognizerConfig,
    audio: AudioInput,
    async_handle: Option<RecognizerAsync>,
    timeout: u32,
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
                myself: Weak::new(),
                hooks: EventFlags::default(),
                subscribers: Vec::new(),
                async_handle: None,
                timeout: UINT32_MAX,
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

    pub fn is_started(&self) -> bool {
        self.async_handle.is_some()
    }

    pub fn start(am: &Arc<Mutex<Self>>) -> Result {
        let myself = Arc::clone(am);
        let myself = Arc::downgrade(&myself);
        let mut me = am.try_lock()?;
        if me.is_started() {
            return Err(SpxError::AlreadyExists);
        }
        let mut h = null_mut();
        unsafe {
            hr(recognizer_start_continuous_recognition_async(
                me.handle, &mut h,
            ))?;
            hr(recognizer_start_continuous_recognition_async_wait_for(
                h, me.timeout,
            ))?;
        }
        let ah = RecognizerAsync::new(h);
        me.async_handle = Some(ah);
        me.myself = myself;
        me.hook(me.hooks)
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

    fn hook(&mut self, flags: EventFlags) -> Result {
        let mut me = self.myself.clone();
        let vp: *mut c_void = &mut me as *mut _ as *mut c_void;
        if flags.contains(EventFlags::Recognizing) {
            hr(unsafe {
                recognizer_recognizing_set_callback(
                    self.handle,
                    Some(on_recognizing),
                    vp,
                )
            })?;
        }

        if flags.contains(EventFlags::Recognized) {
            hr(unsafe {
                recognizer_recognized_set_callback(
                    self.handle,
                    Some(on_recognized),
                    vp,
                )
            })?;
        }
        Ok(())
    }

    fn on_session_started(&mut self, evt: RecognizerEvent) -> Result {
        let id = evt.id()?;
        let offset = evt.offset()?;
        let mut started = AsrEvent::new(EventFlags::SessionStarted);
        started.id = Some(id);
        started.offset = Some(offset);
        Ok(())
    }
}

DeriveSpxHandle!(
    Event,
    recognizer_event_handle_release,
    recognizer_event_handle_is_valid
);

pub struct Event {
    flag: EventFlags,
    handle: SPXEVENTHANDLE,
}

impl Event {
    pub fn flag(&self) -> EventFlags {
        self.flag
    }
    
    pub fn id(&self) -> Result<String> {
        get_cf_string(recognizer_session_event_get_session_id, self.handle, 40)
    }

    pub fn offset(&self) -> Result<u64> {
        let mut offset = 0;
        hr(unsafe {
            recognizer_recognition_event_get_offset(self.handle, &mut offset)
        })?;
        Ok(offset)
    }

    // pub fn extract(&self, tag: EventFlags) -> Result<AsrEvent> {
    //     let id = get_cf_string(recognizer_session_event_get_session_id, self.handle, 40)?;
    //     let mut offset = 0;
    //     hr(unsafe {
    //         recognizer_recognition_event_get_offset(self.handle, &mut offset)
    //     })?;
    //     let mut results = None;
    //     let id = Some(id);
    //     let offset = Some(offset);
    //     if tag.intersects(EventFlags::Session) {
    //         Ok(AsrEvent {tag, id, offset, results})
    //     } else if tag.intersects(EventFlags::)
    // }
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

DefCallback!(on_session_started, EventFlags::SessionStarted);
// DefCallback!(on_session_stopped);
// DefCallback!(on_speech_start);
// DefCallback!(on_speech_end);
// DefCallback!(on_recognizing);
// DefCallback!(on_recognized);
// DefCallback!(on_connected);
// DefCallback!(on_disconnected);
