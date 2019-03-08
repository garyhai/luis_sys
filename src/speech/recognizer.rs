//! Recognizer for speech with intent and translation support.

use super::{
    audio::AudioStream,
    events::{
        AsrResult, CancellationResult, Event, EventResult, Flags, Recognition,
        RecognitionResult, Session,
    },
};
use crate::{
    hr, speech_api::*, DeriveHandle, Handle, Result, SmartHandle, SpxError,
    INVALID_HANDLE,
};
use futures::{
    sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    try_ready, Async, Poll, Stream,
};
use std::{
    ffi::CString,
    os::raw::c_void,
    ptr::null,
    sync::{Arc, Weak},
};

/// The event callback definition macro.
macro_rules! DefCallback {
    ($name:ident, $flag:expr) => {
        #[no_mangle]
        unsafe extern "C" fn $name(
            _: SPXRECOHANDLE,
            hevent: SPXEVENTHANDLE,
            context: *mut c_void,
        ) {
            fire_on_event($flag, hevent, context);
        }
    };
}

SmartHandle!(
    RecognizerAsync,
    SPXASYNCHANDLE,
    recognizer_async_handle_release,
    recognizer_async_handle_is_valid
);

SmartHandle!(
    RecognizerSession,
    SPXSESSIONHANDLE,
    session_handle_release,
    session_handle_is_valid
);

SmartHandle!(
    IntentTrigger,
    SPXTRIGGERHANDLE,
    intent_trigger_handle_release,
    intent_trigger_handle_is_valid
);

impl IntentTrigger {
    /// Phrase as singleton intent.
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let phrase = CString::new(phrase)?;
        hr!(intent_trigger_create_from_phrase(
            &mut handle,
            phrase.as_ptr()
        ))?;
        Ok(IntentTrigger::new(handle))
    }

    /// Load intent from language understanding model.
    pub fn from_model(model: &Model, name: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let name = CString::new(name)?;
        hr!(intent_trigger_create_from_language_understanding_model(
            &mut handle,
            model.handle(),
            name.as_ptr()
        ))?;
        Ok(IntentTrigger::new(handle))
    }

    /// Load all intents from the model.
    pub fn from_model_all(model: &Model) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(intent_trigger_create_from_language_understanding_model(
            &mut handle,
            model.handle(),
            null()
        ))?;
        Ok(IntentTrigger::new(handle))
    }
}

SmartHandle!(
    Model,
    SPXLUMODELHANDLE,
    language_understanding_model__handle_release,
    language_understanding_model_handle_is_valid
);

impl Model {
    /// Creates a language understanding model using the specified endpoint url.
    pub fn from_uri(uri: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let uri = CString::new(uri)?;
        hr!(language_understanding_model_create_from_uri(
            &mut handle,
            uri.as_ptr()
        ))?;
        Ok(Model::new(handle))
    }

    /// Creates a language understanding model using the specified app id.
    pub fn from_app_id(id: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let id = CString::new(id)?;
        hr!(language_understanding_model_create_from_app_id(
            &mut handle,
            id.as_ptr()
        ))?;
        Ok(Model::new(handle))
    }

    /// Creates a language understanding model using the specified hostname, subscription key and application id.
    pub fn from_subscription(
        key: &str,
        id: &str,
        region: &str,
    ) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let key = CString::new(key)?;
        let id = CString::new(id)?;
        let region = CString::new(region)?;
        hr!(language_understanding_model_create_from_subscription(
            &mut handle,
            key.as_ptr(),
            id.as_ptr(),
            region.as_ptr()
        ))?;
        Ok(Model::new(handle))
    }
}

DeriveHandle!(
    Recognizer,
    SPXRECOHANDLE,
    recognizer_handle_release,
    recognizer_handle_is_valid
);

/// In addition to performing speech-to-text recognition, the IntentRecognizer extracts structured information
/// about the intent of the speaker, which can be used to drive further actions using dedicated intent triggers
pub struct Recognizer {
    handle: SPXRECOHANDLE,
    flags: Flags,
    stream: Option<AudioStream>,
    sink: Option<Arc<UnboundedSender<Event>>>,
    timeout: u32,
}

impl Recognizer {
    /// Constructor.
    pub fn new(
        handle: SPXRECOHANDLE,
        stream: Option<AudioStream>,
        flags: Flags,
        timeout: u32,
    ) -> Self {
        Recognizer {
            handle,
            flags,
            stream,
            timeout,
            sink: None,
        }
    }

    /// Proxy the write function of push stream.
    pub fn write_stream(&self, buffer: &mut [u8]) -> Result {
        let stream = self.stream.as_ref().ok_or(SpxError::NulError)?;
        stream.write(buffer)
    }

    /// Close the push stream gracefully.
    pub fn close_stream(&self) -> Result {
        let stream = self.stream.as_ref().ok_or(SpxError::NulError)?;
        stream.close()
    }

    /// Blocked mode for once recognition.
    pub fn recognize(&self) -> Result<String> {
        let mut hres = INVALID_HANDLE;
        hr!(recognizer_recognize_once(self.handle, &mut hres))?;
        let rr = EventResult::new(Flags::empty(), hres)?;
        let reason = rr.reason();
        if reason.contains(Flags::Recognized) {
            Ok(String::from(rr.text()?))
        } else if reason.contains(Flags::Canceled) {
            rr.cancellation_error()
        } else {
            Err(SpxError::Unknown(format!("unhandled reason {:?}", reason)))
        }
    }

    /// Pause the progress of recognition.
    pub fn pause(&self) -> Result {
        hr!(recognizer_disable(self.handle))
    }

    /// Resume paused session.
    pub fn resume(&self) -> Result {
        hr!(recognizer_enable(self.handle))
    }

    /// Check started by event sink handle.
    pub fn started(&self) -> bool {
        self.sink.is_some()
    }

    /// Start the recognition session with configuration present.
    pub fn start(&mut self) -> Result<EventStream> {
        // Flags::Session | Flags::Canceled is set default. But may not be dicovered by stream filter.
        self.start_flags(Flags::Session | Flags::Canceled)
    }

    /// Stop the sesstion.
    pub fn stop(&mut self) -> Result {
        let mut h = INVALID_HANDLE;
        hr!(recognizer_stop_continuous_recognition_async(
            self.handle,
            &mut h,
        ))?;
        let _ = RecognizerAsync::new(h);
        self.sink = None;
        Ok(())
    }

    /// Start recognition with customized flags.
    /// Notice: If Flags::Cancled is not set, error message may not be handled; If Flags::Session is not set, stream future may not be resolved.
    pub fn start_flags(&mut self, flags: Flags) -> Result<EventStream> {
        if self.started() {
            return Err(SpxError::AlreadyExists);
        }

        let flags = self.flags | flags;
        let mut h = INVALID_HANDLE;
        hr!(recognizer_start_continuous_recognition_async(
            self.handle,
            &mut h,
        ))?;
        let _ra = RecognizerAsync::new(h);
        hr!(recognizer_start_continuous_recognition_async_wait_for(
            h,
            self.timeout,
        ))?;

        let (s, r) = unbounded::<Event>();
        self.sink = Some(Arc::new(s));
        let reception = EventStream::new(r, flags);

        let sink = self.sink.as_mut().unwrap();
        let sk = Box::new(Arc::downgrade(sink));
        let context = Box::into_raw(sk) as *mut c_void;

        if flags.contains(Flags::Recognizing) {
            hr!(recognizer_recognizing_set_callback(
                self.handle,
                Some(on_recognizing),
                context,
            ))?;
        }

        if flags.contains(Flags::Recognized) {
            hr!(recognizer_recognized_set_callback(
                self.handle,
                Some(on_recognized),
                context,
            ))?;
        }

        if flags.contains(Flags::SessionStarted) {
            hr!(recognizer_session_started_set_callback(
                self.handle,
                Some(on_session_started),
                context,
            ))?;
        }

        if flags.contains(Flags::SessionStopped) {
            hr!(recognizer_session_stopped_set_callback(
                self.handle,
                Some(on_session_stopped),
                context,
            ))?;
        }

        if flags.contains(Flags::SpeechStartDetected) {
            hr!(recognizer_speech_start_detected_set_callback(
                self.handle,
                Some(on_speech_start),
                context,
            ))?;
        }

        if flags.contains(Flags::SpeechEndDetected) {
            hr!(recognizer_speech_end_detected_set_callback(
                self.handle,
                Some(on_speech_end),
                context,
            ))?;
        }

        let mut h_conn = INVALID_HANDLE;
        hr!(connection_from_recognizer(self.handle, &mut h_conn))?;
        if flags.contains(Flags::Connected) {
            hr!(connection_connected_set_callback(
                h_conn,
                Some(on_connected),
                context,
            ))?;
        }

        if flags.contains(Flags::Disconnected) {
            hr!(connection_disconnected_set_callback(
                h_conn,
                Some(on_disconnected),
                context,
            ))?;
        }
        hr!(connection_handle_release(h_conn))?;

        if flags.contains(Flags::Canceled) {
            hr!(recognizer_canceled_set_callback(
                self.handle,
                Some(on_canceled),
                context,
            ))?;
        }

        Ok(reception)
    }

    /// Add generated intent trigger.
    pub fn add_intent(&self, id: &str, trigger: &IntentTrigger) -> Result {
        if id.is_empty() {
            hr!(intent_recognizer_add_intent(
                self.handle,
                null(),
                trigger.handle()
            ))
        } else {
            let id = CString::new(id)?;
            hr!(intent_recognizer_add_intent(
                self.handle,
                id.as_ptr(),
                trigger.handle()
            ))
        }
    }
}

/// Promise of recognition event stream.
pub struct EventStream {
    filter: Flags,
    source: UnboundedReceiver<Event>,
    stopped: bool,
}

impl EventStream {
    /// Constructor with filter.
    pub fn new(source: UnboundedReceiver<Event>, filter: Flags) -> Self {
        EventStream {
            filter,
            source,
            stopped: false,
        }
    }

    /// Define the new filter to pick out special events.
    pub fn set_filter(mut self, flags: Flags) -> Self {
        self.filter = flags;
        self
    }

    /// Result streaming of event object.
    pub fn resulting(
        self,
    ) -> impl Stream<Item = Recognition, Error = SpxError> {
        self.then(|res| {
            if let Ok(evt) = res {
                evt.into_result()
            } else {
                Err(SpxError::Unknown(String::from("streaming is interrupted")))
            }
        })
    }

    /// Convert the event object streamto JSON string stream.
    pub fn json(self) -> impl Stream<Item = String, Error = String> {
        self.resulting().then(|res| match res {
            Ok(v) => serde_json::to_string(&v).map_err(|err| err.to_string()),
            Err(v) => Err(serde_json::to_string(&v)
                .map_err(|err| err.to_string())
                .expect("unexpected")),
        })
    }

    /// Speech recognition text only.
    pub fn text(self) -> impl Stream<Item = String, Error = SpxError> {
        let this = self.set_filter(Flags::Recognized);
        this.resulting().map(|reco| reco.text_only())
    }
}

/// The streaming implementation of futures.
impl Stream for EventStream {
    type Item = Event;
    type Error = ();
    fn poll(&mut self) -> Poll<Option<Event>, ()> {
        while !self.stopped {
            match try_ready!(self.source.poll()) {
                Some(evt) => {
                    if evt.flag().contains(Flags::SessionStopped) {
                        self.stopped = true;
                    }
                    if evt.flag().intersects(self.filter) {
                        return Ok(Async::Ready(Some(evt)));
                    }
                }
                None => return Ok(Async::Ready(None)),
            }
        }
        Ok(Async::Ready(None))
    }
}

DefCallback!(on_recognizing, Flags::Recognizing);
DefCallback!(on_recognized, Flags::Recognized);
DefCallback!(on_session_started, Flags::SessionStarted);
DefCallback!(on_session_stopped, Flags::SessionStopped);
DefCallback!(on_speech_start, Flags::SpeechStartDetected);
DefCallback!(on_speech_end, Flags::SpeechEndDetected);
DefCallback!(on_canceled, Flags::Canceled);

#[no_mangle]
unsafe extern "C" fn on_connected(
    hevent: SPXEVENTHANDLE,
    context: *mut c_void,
) {
    fire_on_event(Flags::Connected, hevent, context);
}

#[no_mangle]
unsafe extern "C" fn on_disconnected(
    hevent: SPXEVENTHANDLE,
    context: *mut c_void,
) {
    fire_on_event(Flags::Disconnected, hevent, context);
}

fn fire_on_event(flag: Flags, hevent: SPXEVENTHANDLE, context: *mut c_void) {
    let evt = Event::new(flag, hevent);
    if context.is_null() {
        log::error!("Unknown context with NULL pointer.");
        return;
    }
    log::trace!("Event is fired with {:?} and address: {:?}", flag, context);
    let ctx =
        unsafe { Box::from_raw(context as *mut Weak<UnboundedSender<Event>>) };
    let weak_ptr = Weak::clone(&ctx);
    // forget the box, at least one box is leaked.
    Box::into_raw(ctx);
    if let Some(mut arc) = weak_ptr.upgrade() {
        let sender = Arc::make_mut(&mut arc);
        if let Err(err) = sender.unbounded_send(evt) {
            log::error!("failed to post {:?} event: {}", flag, err);
        }
    } else {
        log::error!("Recognizer instance is dropped!");
    }
}
