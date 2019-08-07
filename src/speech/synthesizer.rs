//! Synthesizer for text to speech with streaming support.

use super::{
    audio::{Audio, AudioStream},
    events::{Event, Flags, Recognition, Session, SynthEventResult},
};

use crate::{
    error::{AlreadyExists, Other, SpxError},
    hr,
    properties::Properties,
    speech_api::{
        synthesizer_async_handle_is_valid, synthesizer_async_handle_release,
        synthesizer_canceled_set_callback, synthesizer_completed_set_callback,
        synthesizer_disable, synthesizer_enable, synthesizer_get_property_bag,
        synthesizer_handle_is_valid, synthesizer_handle_release,
        synthesizer_speak_ssml, synthesizer_speak_ssml_async,
        synthesizer_speak_text, synthesizer_speak_text_async,
        synthesizer_start_speaking_ssml_async,
        synthesizer_start_speaking_text_async,
        synthesizer_started_set_callback,
        synthesizer_synthesizing_set_callback, SPXASYNCHANDLE, SPXEVENTHANDLE,
        SPXSYNTHHANDLE,
    },
    DeriveHandle, FlattenProps, Result, SmartHandle, INVALID_HANDLE,
};

use futures::{
    sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    try_ready, Async, Poll, Stream,
};

use std::{
    ffi::CString,
    os::raw::c_void,
    sync::{Arc, Weak},
};

/// The event callback definition macro.
macro_rules! DefCallback {
    ($name:ident, $flag:expr) => {
        #[no_mangle]
        unsafe extern "C" fn $name(
            _: SPXSYNTHHANDLE,
            hevent: SPXEVENTHANDLE,
            context: *mut c_void,
        ) {
            fire_on_event($flag, hevent, context);
        }
    };
}

SmartHandle!(
    SynthesizerAsync,
    SPXASYNCHANDLE,
    synthesizer_async_handle_release,
    synthesizer_async_handle_is_valid
);

DeriveHandle!(
    Synthesizer,
    SPXSYNTHHANDLE,
    synthesizer_handle_release,
    synthesizer_handle_is_valid
);

/// Synthesizer support many ways of audio output.
pub struct Synthesizer {
    handle: SPXSYNTHHANDLE,
    flags: Flags,
    audio: Audio,
    sink: Option<Arc<UnboundedSender<Event>>>,
    /// Internal properties bag.
    props: Properties,
}
FlattenProps!(Synthesizer);

impl Synthesizer {
    /// Constructor.
    pub fn new(
        handle: SPXSYNTHHANDLE,
        audio: Audio,
        flags: Flags,
    ) -> Result<Self> {
        let mut hprops = INVALID_HANDLE;
        hr!(synthesizer_get_property_bag(handle, &mut hprops))?;
        Ok(Synthesizer {
            handle,
            flags,
            audio,
            sink: None,
            props: Properties::new(hprops),
        })
    }

    /// Execute the speech synthesis on plain text, asynchronously.
    pub fn synthesize(&mut self, text: &str) -> Result {
        let mut hasync = INVALID_HANDLE;
        let txt_len = text.len() as u32;
        let txt = CString::new(text)?.as_ptr();
        hr!(synthesizer_speak_text_async(
            self.handle,
            txt,
            txt_len,
            &mut hasync
        ))?;
        SynthesizerAsync::new(hasync);
        Ok(())
    }

    /// Execute the speech synthesis on SSML, synchronously.
    pub fn ssml_synthesis(&mut self, text: &str) -> Result {
        let mut hasync = INVALID_HANDLE;
        let txt_len = text.len() as u32;
        let txt = CString::new(text)?.as_ptr();
        hr!(synthesizer_speak_ssml_async(
            self.handle,
            txt,
            txt_len,
            &mut hasync
        ))?;
        SynthesizerAsync::new(hasync);
        Ok(())
    }

    /// Execute the speech synthesis on plain text, asynchronously.
    pub fn start_synthesize(&mut self, text: &str) -> Result {
        let mut hasync = INVALID_HANDLE;
        let txt_len = text.len() as u32;
        let txt = CString::new(text)?.as_ptr();
        hr!(synthesizer_start_speaking_text_async(
            self.handle,
            txt,
            txt_len,
            &mut hasync
        ))?;
        SynthesizerAsync::new(hasync);
        Ok(())
    }

    /// Execute the speech synthesis on SSML, asynchronously.
    pub fn start_ssml_synthesis(&mut self, text: &str) -> Result {
        let mut hasync = INVALID_HANDLE;
        let txt_len = text.len() as u32;
        let txt = CString::new(text)?.as_ptr();
        hr!(synthesizer_start_speaking_ssml_async(
            self.handle,
            txt,
            txt_len,
            &mut hasync
        ))?;
        SynthesizerAsync::new(hasync);
        Ok(())
    }

    /// Execute the speech synthesis on plain text, synchronously.
    pub fn synthesis_once(&mut self, text: &str) -> Result<SynthEventResult> {
        let mut hres = INVALID_HANDLE;
        let txt_len = text.len() as u32;
        let txt = CString::new(text)?.as_ptr();
        hr!(synthesizer_speak_text(self.handle, txt, txt_len, &mut hres))?;
        SynthEventResult::new(Flags::empty(), hres)
    }

    /// Execute the speech synthesis on SSML, synchronously.
    pub fn ssml_synthesis_once(
        &mut self,
        text: &str,
    ) -> Result<SynthEventResult> {
        let mut hres = INVALID_HANDLE;
        let txt_len = text.len() as u32;
        let txt = CString::new(text)?.as_ptr();
        hr!(synthesizer_speak_ssml(self.handle, txt, txt_len, &mut hres))?;
        SynthEventResult::new(Flags::empty(), hres)
    }

    /// Input audio data via created stream.
    pub fn write_stream(&mut self, buffer: &mut [u8]) -> Result {
        self.audio.write(buffer)
    }

    /// Output audio data via created stream.
    pub fn read_stream(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.audio.read(buffer)
    }

    /// Close the audio stream gracefully.
    pub fn close_stream(&mut self) -> Result {
        self.audio.close()
    }

    /// Pause the progress of recognition.
    pub fn pause(&self) -> Result {
        hr!(synthesizer_disable(self.handle))
    }

    /// Resume paused session.
    pub fn resume(&self) -> Result {
        hr!(synthesizer_enable(self.handle))
    }

    /// Check started by event sink handle.
    pub fn started(&self) -> bool {
        self.sink.is_some()
    }

    /// Start the synthesis session with configuration present.
    pub fn start(&mut self) -> Result<EventStream> {
        // Flags::SessionStarted | Flags::Canceled is set default. But should not be unseted by stream filter.
        self.start_flags(Flags::SessionStarted | Flags::Canceled)
    }

    /// Stop the sesstion.
    pub fn stop(&mut self) -> Result {
        self.close_stream()?;
        self.sink = None;
        Ok(())
    }

    /// Start recognition with customized flags.
    /// Notice: If Flags::Cancled is not set, error message may not be handled; If Flags::Session is not set, stream future may not be resolved.
    pub fn start_flags(&mut self, flags: Flags) -> Result<EventStream> {
        if self.started() {
            return Err(AlreadyExists);
        }

        let flags = self.flags | flags;

        let (s, r) = unbounded::<Event>();
        let sink = Arc::new(s);
        self.sink = Some(sink.clone());
        let reception = EventStream::new(r, flags);

        let sk = Box::new(Arc::downgrade(&sink));
        let context = Box::into_raw(sk) as *mut c_void;

        if flags.contains(Flags::Synthesizing) {
            hr!(synthesizer_synthesizing_set_callback(
                self.handle,
                Some(on_synthesizing),
                context,
            ))?;
        }

        if flags.contains(Flags::Synthesized) {
            hr!(synthesizer_completed_set_callback(
                self.handle,
                Some(on_synthesized),
                context,
            ))?;
        }

        if flags.contains(Flags::SessionStarted) {
            hr!(synthesizer_started_set_callback(
                self.handle,
                Some(on_synthesis_started),
                context,
            ))?;
        }

        if flags.contains(Flags::Canceled) {
            hr!(synthesizer_canceled_set_callback(
                self.handle,
                Some(on_synth_canceled),
                context,
            ))?;
        }

        Ok(reception)
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
                Err(Other(String::from("streaming is interrupted")))
            }
        })
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
                    if evt
                        .flag()
                        // .intersects(Flags::Synthesized | Flags::Canceled)
                        .intersects(Flags::Canceled)
                    {
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

DefCallback!(on_synthesizing, Flags::Synthesizing);
DefCallback!(on_synthesized, Flags::Synthesized);
DefCallback!(on_synthesis_started, Flags::SessionStarted);
DefCallback!(on_synth_canceled, Flags::Canceled);

fn fire_on_event(flag: Flags, hevent: SPXEVENTHANDLE, context: *mut c_void) {
    log::trace!("Recognition event {:?} fired.", flag);
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
