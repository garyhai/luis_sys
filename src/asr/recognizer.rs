use super::events::*;
use crate::{
    hr, speech_api::*, DeriveHandle, Result, SmartHandle, SpxError,
    INVALID_HANDLE,
};
use futures::{
    sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    Poll, Stream,
};
use std::{
    os::raw::c_void,
    sync::{Arc, Weak},
};

macro_rules! DefCallback {
    ($name:ident, $flag:expr) => {
        // #[no_mangle]
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
    sink: Option<Arc<UnboundedSender<Event>>>,
    reception: Option<Arc<UnboundedReceiver<Event>>>,
    flags: Flags,
    timeout: u32,
    asyn_handle: Option<RecognizerAsync>,
}

impl Recognizer {
    pub fn new(handle: SPXRECOHANDLE, flags: Flags, timeout: u32) -> Self {
        Recognizer {
            handle,
            flags,
            timeout,
            sink: None,
            reception: None,
            asyn_handle: None,
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

    pub fn start(
        &mut self,
        flags: Flags,
    ) -> Result<Arc<UnboundedReceiver<Event>>> {
        if self.started() {
            return Err(SpxError::AlreadyExists);
        }
        self.flags |= flags;
        self.hook(self.flags)?;
        Ok(Arc::clone(&self.reception.as_ref().unwrap()))
    }

    pub fn stop(&mut self) -> Result {
        let mut h = INVALID_HANDLE;
        hr!(recognizer_stop_continuous_recognition_async(
            self.handle,
            &mut h,
        ))?;
        self.asyn_handle = Some(RecognizerAsync::new(h));
        self.sink = None;
        self.reception = None;
        Ok(())
    }

    pub fn resulting(&mut self) -> Result<Arc<UnboundedReceiver<Event>>> {
        if self.started() {
            Ok(Arc::clone(self.reception.as_ref().unwrap()))
        } else {
            self.start(Flags::empty())
        }
    }

    fn hook(&mut self, flags: Flags) -> Result {
        let mut h = INVALID_HANDLE;
        hr!(recognizer_start_continuous_recognition_async(
            self.handle,
            &mut h,
        ))?;
        self.asyn_handle = Some(RecognizerAsync::new(h));
        hr!(recognizer_start_continuous_recognition_async_wait_for(
            h,
            self.timeout,
        ))?;
        if self.sink.is_none() {
            let (s, r) = unbounded::<Event>();
            self.sink = Some(Arc::new(s));
            self.reception = Some(Arc::new(r));
        }
        let sink = self.sink.as_mut().unwrap();
        let mut sink = Arc::downgrade(&sink);
        let context: *mut c_void = &mut sink as *mut _ as *mut c_void;
        if flags.contains(Flags::Recognizing) {
            hr!(recognizer_recognizing_set_callback(
                self.handle,
                Some(on_recognizing),
                context,
            ))?;
        }

        if flags.contains(Flags::Recognized) {
            hr!(recognizer_recognizing_set_callback(
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

        Ok(())
    }
}

impl Stream for Recognizer {
    type Item = Event;
    type Error = SpxError;
    fn poll(&mut self) -> Poll<Option<Event>, SpxError> {
        if self.started() {
            let ar = self.reception.as_mut().unwrap();
            if let Some(ref mut r) = Arc::get_mut(ar) {
                r.poll().map_err(|_| SpxError::Unknown(String::new()))
            } else {
                Err(SpxError::Unknown(String::from("Resulting is locked")))
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
// DefCallback!(on_speech_start);
// DefCallback!(on_speech_end);
// DefCallback!(on_connected);
// DefCallback!(on_disconnected);

fn fire_on_event(flag: Flags, hevent: SPXEVENTHANDLE, context: *mut c_void) {
    let evt = Event::new(flag, hevent);
    if context.is_null() {
        log::error!("Unknown context with NULL pointer.");
        return;
    }
    log::trace!(
        "Event is fired with flag: {:?} and address: {:?}",
        flag,
        context
    );
    let context =
        unsafe { &mut *(context as *mut Weak<UnboundedSender<Event>>) };
    let context = Weak::clone(context);
    if let Some(mut arc) = context.upgrade() {
        let sender = Arc::make_mut(&mut arc);
        if let Err(err) = sender.unbounded_send(evt) {
            log::error!("failed to post event data by error: {}", err);
            log::debug!("{:?}", err);
        }
    } else {
        log::error!("Recognizer instance is dropped!");
    }
}
