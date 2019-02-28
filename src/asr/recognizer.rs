use super::events::*;
use crate::{
    hr, speech_api::*, DeriveHandle, Result, SmartHandle, SpxError,
    INVALID_HANDLE,
};
use futures::{
    sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    Poll, Stream, Async
};
use std::{
    mem,
    os::raw::c_void,
    sync::{Arc, Weak},
};

pub type Msg = Result<Recognition>;

macro_rules! DefCallback {
    ($name:ident, $flag:expr) => {
        #[no_mangle]
        unsafe extern "C" fn $name(
            _: SPXRECOHANDLE,
            hevent: SPXEVENTHANDLE,
            context: *mut c_void,
        ) {
            // log::warn!("h_reco: {}, hevent: {}, flag: {:?}", h as usize, hevent as usize, $flag);
            // let evt = Event::new($flag, hevent);
            // log::warn!("event: {:?}", evt.into_result());
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
    sink: Option<Arc<UnboundedSender<Msg>>>,
    reception: Option<UnboundedReceiver<Msg>>,
    flags: Flags,
    timeout: u32,
}

impl Recognizer {
    pub fn new(handle: SPXRECOHANDLE, flags: Flags, timeout: u32) -> Self {
        Recognizer {
            handle,
            flags,
            timeout,
            sink: None,
            reception: None,
        }
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

    pub fn start(&mut self) -> Result {
        self.start_flags(Flags::empty())
    }

    pub fn start_flags(&mut self, flags: Flags) -> Result {
        if self.started() {
            return Err(SpxError::AlreadyExists);
        }
        self.flags |= flags;
        self.hook(self.flags)
    }

    pub fn stop(&mut self) -> Result {
        let mut h = INVALID_HANDLE;
        hr!(recognizer_stop_continuous_recognition_async(
            self.handle,
            &mut h,
        ))?;
        let _ = RecognizerAsync::new(h);
        self.sink = None;
        self.reception = None;
        Ok(())
    }

    fn hook(&mut self, flags: Flags) -> Result {
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

        if self.sink.is_none() {
            let (s, r) = unbounded::<Msg>();
            self.sink = Some(Arc::new(s));
            self.reception = Some(r);
        }
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

        Ok(())
    }
}

impl Stream for Recognizer {
    type Item = Recognition;
    type Error = SpxError;
    fn poll(&mut self) -> Poll<Option<Recognition>, SpxError> {
        if let Some(ref mut r) = self.reception.as_mut() {
            match r.poll() {
                Ok(Async::Ready(Some(Ok(msg)))) => Ok(Async::Ready(Some(msg))),
                Ok(Async::Ready(Some(Err(err)))) => Err(err),
                Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(_) => Err(SpxError::Unknown(String::new())),
            }
        } else {
            Err(SpxError::Unknown(String::from("Recognizer is not started")))
        }
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
    // log::warn!("hevent: {}, flag: {:?}", hevent as usize, Flags::Connected);
    fire_on_event(Flags::Connected, hevent, context);
}

#[no_mangle]
unsafe extern "C" fn on_disconnected(
    hevent: SPXEVENTHANDLE,
    context: *mut c_void,
) {
    // log::warn!("hevent: {}, flag: {:?}", hevent as usize, Flags::Disconnected);
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
        unsafe { Box::from_raw(context as *mut Weak<UnboundedSender<Msg>>) };
    let weak_ptr = Weak::clone(&ctx);
    mem::forget(ctx);
    if let Some(mut arc) = weak_ptr.upgrade() {
        let sender = Arc::make_mut(&mut arc);
        if let Err(err) = sender.unbounded_send(evt.into_result()) {
            log::error!("failed to post event data by error: {}", err);
            log::debug!("{:?}", err);
        }
    } else {
        log::error!("Recognizer instance is dropped!");
    }
}
