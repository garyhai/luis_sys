use super::events::{
    AsrResult, Event, EventResult, Flags, Recognition, RecognitionResult,
    Session,
};
use crate::{
    audio::AudioStream,
    hr,
    speech_api::{
        connection_connected_set_callback,
        connection_disconnected_set_callback, connection_from_recognizer,
        connection_handle_release, recognizer_async_handle_is_valid,
        recognizer_async_handle_release, recognizer_canceled_set_callback,
        recognizer_disable, recognizer_enable, recognizer_handle_is_valid,
        recognizer_handle_release, recognizer_recognize_once,
        recognizer_recognized_set_callback,
        recognizer_recognizing_set_callback,
        recognizer_session_started_set_callback,
        recognizer_session_stopped_set_callback,
        recognizer_speech_end_detected_set_callback,
        recognizer_speech_start_detected_set_callback,
        recognizer_start_continuous_recognition_async,
        recognizer_start_continuous_recognition_async_wait_for,
        recognizer_stop_continuous_recognition_async, session_handle_is_valid,
        session_handle_release, Result_Reason_ResultReason_RecognizedSpeech,
        SPXASYNCHANDLE, SPXEVENTHANDLE, SPXRECOHANDLE, SPXSESSIONHANDLE,
    },
    DeriveHandle, Result, SmartHandle, SpxError, INVALID_HANDLE,
};
use futures::{
    sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    try_ready, Async, Poll, Stream,
};
use std::{
    os::raw::c_void,
    sync::{Arc, Weak},
};

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

DeriveHandle!(
    Recognizer,
    SPXRECOHANDLE,
    recognizer_handle_release,
    recognizer_handle_is_valid
);

pub struct Recognizer {
    handle: SPXRECOHANDLE,
    flags: Flags,
    stream: Option<AudioStream>,
    sink: Option<Arc<UnboundedSender<Event>>>,
    timeout: u32,
}

impl Recognizer {
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

    pub fn write_stream(&self, buffer: &mut [u8]) -> Result {
        let stream = self.stream.as_ref().ok_or(SpxError::NulError)?;
        stream.write(buffer)
    }

    pub fn close_stream(&self) -> Result {
        let stream = self.stream.as_ref().ok_or(SpxError::NulError)?;
        stream.close()
    }

    pub fn recognize(&self) -> Result<String> {
        let mut hres = INVALID_HANDLE;
        hr!(recognizer_recognize_once(self.handle, &mut hres))?;
        let rr = EventResult::from_handle(hres)?;
        if rr.reason()? == Result_Reason_ResultReason_RecognizedSpeech {
            Ok(String::from(rr.text()?))
        } else {
            Err(SpxError::Unknown(String::from("unhandled")))
        }
    }

    pub fn pause(&self) -> Result {
        hr!(recognizer_disable(self.handle))
    }

    pub fn resume(&self) -> Result {
        hr!(recognizer_enable(self.handle))
    }

    pub fn started(&self) -> bool {
        self.sink.is_some()
    }

    pub fn start(&mut self) -> Result<EventStream> {
        self.start_flags(Flags::empty())
    }

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
}

pub struct EventStream {
    filter: Flags,
    source: UnboundedReceiver<Event>,
    stopped: bool,
}

impl EventStream {
    pub fn new(source: UnboundedReceiver<Event>, filter: Flags) -> Self {
        EventStream {
            filter,
            source,
            stopped: false,
        }
    }

    pub fn filter(mut self, flags: Flags) -> Self {
        self.filter = flags;
        self
    }

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

    pub fn json(self) -> impl Stream<Item = String, Error = String> {
        self.resulting().then(|res| match res {
            Ok(v) => serde_json::to_string(&v).map_err(|err| err.to_string()),
            Err(v) => Err(serde_json::to_string(&v)
                .map_err(|err| err.to_string())
                .expect("unexpected")),
        })
    }

    pub fn text(self) -> impl Stream<Item = String, Error = SpxError> {
        let this = self.filter(Flags::Recognized);
        this.resulting().map(|reco| reco.text_only())
    }
}

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
