//! Events and results.

#![allow(non_upper_case_globals)]

use crate::{
    get_cf_string, hr,
    properties::{Properties, PropertyBag},
    speech_api::*,
    DeriveHandle, FlattenProps, Handle, Result, SpxError, INVALID_HANDLE,
};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{self, json, Value};
use std::{ffi::CStr, os::raw::c_char, ptr::null_mut, slice, time::Duration};

/// Bitmask for events callbacks and reason of result.
bitflags! {
    #[derive(Default, Deserialize)]
    pub struct Flags: u64 {
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
        const Recognition = 0b1100_0000;
        const Speech = 0b0001_0000_0000;
        const Intent = 0b0010_0000_0000;
        const Translation = 0b0100_0000_0000;
        const Synthesis = 0b1000_0000_0000;
        const Canceled = 0b0001_0000_0000_0000;
        const NoMatch = 0b0010_0000_0000_0000;
    }
}

/// Make output more readable.
impl Serialize for Flags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let display = format!("{:?}", self);
        serializer.serialize_str(&display)
    }
}

/// Convert from underlying integer const of result reason.
impl From<Result_Reason> for Flags {
    fn from(reason: Result_Reason) -> Self {
        match reason {
            Result_Reason_ResultReason_NoMatch => Flags::NoMatch,
            Result_Reason_ResultReason_Canceled => Flags::Canceled,
            Result_Reason_ResultReason_RecognizingSpeech => {
                Flags::Recognizing | Flags::Speech
            }
            Result_Reason_ResultReason_RecognizedSpeech => {
                Flags::Recognized | Flags::Speech
            }
            Result_Reason_ResultReason_RecognizingIntent => {
                Flags::Recognizing | Flags::Intent
            }
            Result_Reason_ResultReason_RecognizedIntent => {
                Flags::Recognized | Flags::Intent
            }
            Result_Reason_ResultReason_TranslatingSpeech => {
                Flags::Recognizing | Flags::Translation
            }
            Result_Reason_ResultReason_TranslatedSpeech => {
                Flags::Recognized | Flags::Translation
            }
            Result_Reason_ResultReason_SynthesizingAudio => {
                Flags::Recognizing | Flags::Synthesis
            }
            Result_Reason_ResultReason_SynthesizingAudioComplete => {
                Flags::Recognized | Flags::Synthesis
            }
            _ => {
                log::error!("Unknown reason to convert Flags!");
                Flags::empty()
            }
        }
    }
}

/// For stringify output.
pub trait ToJson
where
    Self: Serialize + Sized,
{
    fn to_json(self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }

    fn to_json_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Make NoMatch reason readable.
#[derive(Debug, Serialize)]
pub enum Matching {
    Matched,
    NotRecognized,
    InitialSilenceTimeout,
    InitialBabbleTimeout,
}

/// Convert from underlying integer const of no match reason.
impl From<Result_NoMatchReason> for Matching {
    fn from(reason: Result_NoMatchReason) -> Self {
        match reason {
            Result_NoMatchReason_NoMatchReason_NotRecognized => {
                Matching::NotRecognized
            }
            Result_NoMatchReason_NoMatchReason_InitialSilenceTimeout => {
                Matching::InitialSilenceTimeout
            }
            Result_NoMatchReason_NoMatchReason_InitialBabbleTimeout => {
                Matching::InitialBabbleTimeout
            }
            _ => {
                log::error!("Unknown no match reason: {}", reason);
                Matching::NotRecognized
            }
        }
    }
}

/// Output of recognition
#[derive(Debug, Default, Serialize)]
pub struct Recognition {
    pub flag: Flags,
    pub session: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Flags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching: Option<Matching>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translations: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synthesis: Option<Vec<u8>>,
}

impl Recognition {
    /// Get only the speech recognition text.
    pub fn text_only(self) -> String {
        self.text.unwrap_or_else(|| String::new())
    }
}

impl ToJson for Recognition {}

/// Refine the cancellation.
#[derive(Debug, Default, Serialize)]
pub struct CancellationError {
    reason: Result_CancellationReason,
    code: Result_CancellationErrorCode,
    details: String,
}
impl ToJson for CancellationError {}

/// Refine the NoMatch reason.
#[derive(Debug, Serialize)]
pub struct NoMatchError {
    reason: Matching,
}
impl ToJson for NoMatchError {}

DeriveHandle!(
    Event,
    SPXEVENTHANDLE,
    recognizer_event_handle_release,
    recognizer_event_handle_is_valid
);

/// Generic event fired by LUIS engine.
pub struct Event {
    /// Underlying handle.
    handle: SPXEVENTHANDLE,
    /// Flag of the event source.
    flag: Flags,
}

impl Event {
    /// Constructor.
    pub fn new(flag: Flags, handle: SPXEVENTHANDLE) -> Self {
        Event { flag, handle }
    }

    /// Yield the output of event.
    pub fn into_result(self) -> Result<Recognition> {
        let mut r = Recognition::default();
        let flag = self.flag;
        r.flag = flag;
        r.session = self.session_id()?;
        if flag.intersects(Flags::Connection | Flags::Session) {
            return Ok(r);
        }

        if flag.intersects(Flags::SpeechDetection) {
            r.offset = Some(self.offset()?);
            return Ok(r);
        }

        let er = EventResult::from_event(self)?;
        r.id = Some(er.id()?);
        let reason = er.reason();
        r.reason = Some(reason);

        if reason.intersects(Flags::NoMatch) {
            r.matching = Some(er.no_match_reason()?);
            return Ok(r);
        }

        if reason.intersects(Flags::Canceled) {
            if er.code()?
                == Result_CancellationErrorCode_CancellationErrorCode_NoError
            {
                return Ok(r);
            } else {
                return er.cancellation_error();
            }
        }

        if reason.intersects(Flags::Recognition) {
            r.text = Some(er.text()?);
            r.duration = Some(er.duration()?);
            r.offset = Some(er.offset()?);
        }

        if reason.intersects(Flags::Recognized | Flags::Intent) {
            let intent = er.intent()?;
            if !intent.is_empty() {
                r.matching = Some(Matching::Matched);
                r.intent = Some(intent);
            }
            if reason.intersects(Flags::Intent) {
                r.details = Some(er.details()?);
            }
        }

        if reason.intersects(Flags::Recognized | Flags::Translation) {
            let translations = er.translations()?;
            r.translations = Some(translations);
            let sz = er.synthesis_data_length()?;
            if sz != 0 {
                let mut buf = Vec::with_capacity(sz);
                unsafe { buf.set_len(sz) };
                er.synthesis_data(&mut buf)?;
                r.synthesis = Some(buf);
            }
        }

        Ok(r)
    }
}

impl Session for Event {
    fn flag(&self) -> Flags {
        self.flag
    }
}
impl Detection for Event {}

/// Base trait of an event.
pub trait Session: Handle<SPXEVENTHANDLE> {
    /// Flag of callback entry point. One bit flag only.
    fn flag(&self) -> Flags;

    /// UUID of the recognition progress.
    fn session_id(&self) -> Result<String> {
        get_cf_string(
            recognizer_session_event_get_session_id,
            self.handle(),
            40,
        )
    }
}

/// Trait fo speech stop-end detection.
pub trait Detection: Session {
    /// The offset of recognition event
    fn offset(&self) -> Result<Duration> {
        let mut offset = 0;
        hr!(recognizer_recognition_event_get_offset(
            self.handle(),
            &mut offset
        ))?;
        Ok(Duration::from_nanos(offset * 100))
    }
}

DeriveHandle!(
    EventResult,
    SPXRESULTHANDLE,
    recognizer_result_handle_release,
    recognizer_result_handle_is_valid
);

/// Event may produce result or error.
pub struct EventResult {
    reason: Flags,
    handle: SPXRESULTHANDLE,
    props: Properties,
}

impl EventResult {
    /// Create result with event source flag, then patch the flag with reason.
    pub fn new(flag: Flags, handle: SPXRESULTHANDLE) -> Result<Self> {
        let mut reason: Result_Reason = 0;
        hr!(result_get_reason(handle, &mut reason))?;
        let reason = flag | Flags::from(reason);

        let mut hprops = INVALID_HANDLE;
        hr!(result_get_property_bag(handle, &mut hprops))?;
        let props = Properties::new(hprops);
        Ok(EventResult {
            reason,
            handle,
            props,
        })
    }

    /// Consume the Event and create its result.
    pub fn from_event(evt: Event) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(recognizer_recognition_event_get_result(
            evt.handle(),
            &mut handle
        ))?;
        EventResult::new(evt.flag(), handle)
    }
}

FlattenProps!(EventResult);

impl AsrResult for EventResult {
    fn reason(&self) -> Flags {
        self.reason
    }
}
impl RecognitionResult for EventResult {}
impl SpeechResult for EventResult {}
impl IntentResult for EventResult {}
impl TranslationResult for EventResult {}
impl CancellationResult for EventResult {}
impl NoMatchResult for EventResult {}

/// Base trait of speech recognition resuult.
pub trait AsrResult: Handle<SPXRESULTHANDLE> + PropertyBag {
    /// Unique result id.
    fn id(&self) -> Result<String> {
        get_cf_string(result_get_result_id, self.handle(), 40)
    }

    /// Recognition reason by bitmask flags combination.
    fn reason(&self) -> Flags;
}

/// Recognition result with speech position information.
pub trait RecognitionResult: AsrResult {
    /// Convert from duration of recognized speech in ticks.
    /// A single tick represents one hundred nanoseconds or one ten-millionth of a second.
    fn duration(&self) -> Result<Duration> {
        let mut duration = 0u64;
        hr!(result_get_duration(self.handle(), &mut duration))?;
        Ok(Duration::from_nanos(duration * 100))
    }

    /// Convert from offset of the recognized speech in ticks.
    /// A single tick represents one hundred nanoseconds or one ten-millionth of a second.
    fn offset(&self) -> Result<Duration> {
        let mut offset = 0u64;
        hr!(result_get_duration(self.handle(), &mut offset))?;
        Ok(Duration::from_nanos(offset * 100))
    }
}

/// Speech-to-text result.
pub trait SpeechResult: RecognitionResult {
    /// Normalized text generated by a speech recognition engine from recognized input.
    fn text(&self) -> Result<String> {
        get_cf_string(result_get_text, self.handle(), 1024)
    }
}

/// Speech-to-text and intent recognition result.
pub trait IntentResult: SpeechResult {
    /// Unique intent id.
    fn intent(&self) -> Result<String> {
        get_cf_string(intent_result_get_intent_id, self.handle(), 1024)
    }
    /// Details of intent recognition result in JSON format.
    fn details(&self) -> Result<Value> {
        let js = self.get_by_id(
            PropertyId_LanguageUnderstandingServiceResponse_JsonResult,
        )?;
        if js.is_empty() {
            Ok(Value::Null)
        } else {
            Ok(serde_json::from_str(&js)?)
        }
    }
}

/// Defines the translation text result and synthesis result, i.e. the voice output of the translated text in the target language.
pub trait TranslationResult: RecognitionResult {
    /// Presents the translation results. Each item in the map is a key value pair, where key is the language tag of the translated text, and value is the translation text in that language.
    fn translations(&self) -> Result<Value> {
        let mut length = 0;
        hr!(translation_text_result_get_translation_text_buffer_header(
            self.handle(),
            null_mut(),
            &mut length
        ))?;
        if length == 0 {
            return Err(SpxError::NulError);
        }

        let mut buf = Vec::with_capacity(length + 1);
        let header =
            buf.as_mut_ptr() as *mut Result_TranslationTextBufferHeader;
        hr!(translation_text_result_get_translation_text_buffer_header(
            self.handle(),
            header,
            &mut length
        ))?;
        let mut tbl = json!({});
        unsafe {
            let header = &*header;
            let sz = header.numberEntries;
            let ar_lang: &[*mut c_char] =
                slice::from_raw_parts(header.targetLanguages, sz);
            let ar_txt: &[*mut c_char] =
                slice::from_raw_parts(header.translationTexts, sz);
            for i in 0..header.numberEntries {
                let lang = CStr::from_ptr(ar_lang[i]).to_str()?;
                let text = CStr::from_ptr(ar_txt[i]).to_str()?;
                tbl[lang] = json!(text);
            }
        }
        Ok(tbl)
    }

    /// Check the length of synthesised voice data length for allocation of buffer.
    fn synthesis_data_length(&self) -> Result<usize> {
        let mut length = 0;
        hr!(translation_synthesis_result_get_audio_data(
            self.handle(),
            null_mut(),
            &mut length
        ))?;
        Ok(length)
    }

    /// Retrieve the synthesis data from internal buffer by copy data to given buffer.
    fn synthesis_data(&self, buf: &mut [u8]) -> Result {
        let mut length = buf.len();
        let buf_ptr = buf.as_mut_ptr();
        hr!(translation_synthesis_result_get_audio_data(
            self.handle(),
            buf_ptr,
            &mut length
        ))
    }

    /// Simple way to fetch synthesis data.
    fn synthesis(&self) -> Result<Vec<u8>> {
        let length = self.synthesis_data_length()?;
        let mut buf = vec![0; length];
        self.synthesis_data(&mut buf)?;
        Ok(buf)
    }
}

/// Cancellation reason or error.
pub trait CancellationResult: AsrResult {
    /// The reason the result was canceled.
    fn cancellation_reason(&self) -> Result<Result_CancellationReason> {
        let mut n = 0 as Result_CancellationReason;
        hr!(result_get_reason_canceled(self.handle(), &mut n))?;
        Ok(n)
    }

    /// The error code in case of an unsuccessful recognition. If Reason is not Error, ErrorCode is set to NoError.
    fn code(&self) -> Result<Result_CancellationErrorCode> {
        let mut n = 0 as Result_CancellationErrorCode;
        hr!(result_get_canceled_error_code(self.handle(), &mut n))?;
        Ok(n)
    }

    /// The error message in case of an unsuccessful recognition
    fn error_details(&self) -> Result<String> {
        self.get_by_id(PropertyId_SpeechServiceResponse_JsonErrorDetails)
    }

    /// Convert cancellation to SpxError.
    fn cancellation_error<T>(&self) -> Result<T> {
        let reason = self.cancellation_reason()?;
        let code = self.code()?;
        let details = self.error_details()?;
        Err(CancellationError {
            reason,
            code,
            details,
        }
        .into())
    }
}

/// Contains detailed information for NoMatch recognition results.
pub trait NoMatchResult: AsrResult {
    /// The reason the result was not recognized.
    fn no_match_reason(&self) -> Result<Matching> {
        let mut n = 0 as Result_NoMatchReason;
        hr!(result_get_no_match_reason(self.handle(), &mut n))?;
        Ok(Matching::from(n))
    }

    /// Convert no match error to SpxError.
    fn no_match_error<T>(&self) -> Result<T> {
        let reason = self.no_match_reason()?;
        Err(NoMatchError { reason }.into())
    }
}
