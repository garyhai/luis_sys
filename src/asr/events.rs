use crate::{
    get_cf_string, hr,
    properities::{Properties, PropertyBag},
    speech_api::{
        recognizer_event_handle_is_valid, recognizer_event_handle_release,
        recognizer_recognition_event_get_offset,
        recognizer_recognition_event_get_result,
        recognizer_result_handle_is_valid, recognizer_result_handle_release,
        recognizer_session_event_get_session_id,
        result_get_canceled_error_code, result_get_duration,
        result_get_no_match_reason, result_get_property_bag, result_get_reason,
        result_get_reason_canceled, result_get_result_id, result_get_text,
        PropertyId, PropertyId_SpeechServiceResponse_JsonErrorDetails,
        Result_CancellationErrorCode,
        Result_CancellationErrorCode_CancellationErrorCode_NoError,
        Result_CancellationReason, Result_NoMatchReason, Result_Reason,
        Result_Reason_ResultReason_Canceled,
        Result_Reason_ResultReason_NoMatch,
        Result_Reason_ResultReason_RecognizedIntent,
        Result_Reason_ResultReason_RecognizedSpeech,
        Result_Reason_ResultReason_RecognizingIntent,
        Result_Reason_ResultReason_RecognizingSpeech,
        Result_Reason_ResultReason_SynthesizingAudio,
        Result_Reason_ResultReason_SynthesizingAudioComplete,
        Result_Reason_ResultReason_TranslatedSpeech,
        Result_Reason_ResultReason_TranslatingSpeech, SPXEVENTHANDLE,
        SPXRESULTHANDLE,
    },
    DeriveHandle, Handle, Result, SpxError, INVALID_HANDLE,
};
use serde::Serialize;
use serde_json;
use std::time::Duration;

bitflags! {
    #[derive(Default, Serialize)]
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
        const Recognization = 0b1100_0000;
        const Speech = 0b0001_0000_0000;
        const Intent = 0b0010_0000_0000;
        const Translation = 0b0100_0000_0000;
        const Synthesis = 0b1000_0000_0000;
        const Canceled = 0b0001_0000_0000_0000;
        const NoMatch = 0b0010_0000_0000_0000;
    }
}

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

#[derive(Debug, Default, Serialize)]
pub struct Recognition {
    flag: Flags,
    session: String,
    id: Option<String>,
    reason: Option<Result_Reason>,
    offset: Option<Duration>,
    duration: Option<Duration>,
    text: Option<String>,
}

impl ToJson for Recognition {}

#[derive(Debug, Default, Serialize)]
pub struct CancellationError {
    reason: Result_CancellationReason,
    code: Result_CancellationErrorCode,
    details: String,
}
impl ToJson for CancellationError {}

#[derive(Debug, Default, Serialize)]
pub struct NoMatchError {
    reason: Result_NoMatchReason,
}
impl ToJson for NoMatchError {}

DeriveHandle!(
    Event,
    SPXEVENTHANDLE,
    recognizer_event_handle_release,
    recognizer_event_handle_is_valid
);

pub struct Event {
    handle: SPXEVENTHANDLE,
    flag: Flags,
}

impl Event {
    pub fn new(flag: Flags, handle: SPXEVENTHANDLE) -> Self {
        Event { flag, handle }
    }

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
        r.reason = Some(er.reason()?);

        if flag.intersects(Flags::Canceled) {
            if er.code()?
                == Result_CancellationErrorCode_CancellationErrorCode_NoError
            {
                return Ok(r);
            } else {
                return er.cancellation_error();
            }
        }

        if flag.intersects(Flags::Recognization) {
            r.text = Some(er.text()?);
            r.duration = Some(er.duration()?);
            r.offset = Some(er.offset()?);
            return Ok(r);
        }

        Err(SpxError::Unknown(String::from("unknown flag")))
    }
}

impl Session for Event {
    fn flag(&self) -> Flags {
        self.flag
    }

    fn session_id(&self) -> Result<String> {
        get_cf_string(recognizer_session_event_get_session_id, self.handle, 40)
    }
}
impl Detection for Event {}

pub trait Session: Handle<SPXEVENTHANDLE> {
    fn flag(&self) -> Flags;

    fn session_id(&self) -> Result<String> {
        get_cf_string(
            recognizer_session_event_get_session_id,
            self.handle(),
            40,
        )
    }
}

pub trait Detection: Session {
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

pub struct EventResult {
    flag: Flags,
    handle: SPXRESULTHANDLE,
    props: Properties,
}

impl EventResult {
    pub fn new(flag: Flags, handle: SPXRESULTHANDLE) -> Result<Self> {
        let mut hprops = INVALID_HANDLE;
        hr!(result_get_property_bag(handle, &mut hprops))?;
        let props = Properties::new(hprops);
        Ok(EventResult {
            flag,
            handle,
            props,
        })
    }

    pub fn from_event(evt: Event) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(recognizer_recognition_event_get_result(
            evt.handle(),
            &mut handle
        ))?;
        EventResult::new(evt.flag(), handle)
    }

    #[allow(non_upper_case_globals)]
    pub fn from_handle(handle: SPXRESULTHANDLE) -> Result<Self> {
        let mut er = EventResult::new(Flags::empty(), handle)?;
        let flag = match er.reason()? {
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
            _ => Flags::empty(),
        };
        er.flag = flag;
        Ok(er)
    }
}

impl PropertyBag for EventResult {
    fn get_by_id(&self, id: PropertyId) -> Result<String> {
        self.props.get_by_id(id)
    }

    fn get_by_name(&self, name: &str) -> Result<String> {
        self.props.get_by_name(name)
    }

    fn put_by_id(&self, id: PropertyId, value: &str) -> Result<()> {
        self.props.put_by_id(id, value)
    }

    fn put_by_name(&self, name: &str, value: &str) -> Result<()> {
        self.props.put_by_name(name, value)
    }
}

impl AsrResult for EventResult {}
impl RecognitionResult for EventResult {}
impl CancellationResult for EventResult {}
impl NoMatchResult for EventResult {}

pub trait AsrResult: Handle<SPXRESULTHANDLE> + PropertyBag {
    fn id(&self) -> Result<String> {
        get_cf_string(result_get_result_id, self.handle(), 40)
    }

    fn reason(&self) -> Result<Result_Reason> {
        let mut rr = 0;
        hr!(result_get_reason(self.handle(), &mut rr))?;
        Ok(rr)
    }
}

pub trait RecognitionResult: AsrResult {
    fn text(&self) -> Result<String> {
        get_cf_string(result_get_text, self.handle(), 1024)
    }

    fn duration(&self) -> Result<Duration> {
        let mut duration = 0u64;
        hr!(result_get_duration(self.handle(), &mut duration))?;
        Ok(Duration::from_nanos(duration * 100))
    }

    fn offset(&self) -> Result<Duration> {
        let mut offset = 0u64;
        hr!(result_get_duration(self.handle(), &mut offset))?;
        Ok(Duration::from_nanos(offset * 100))
    }
}

pub trait CancellationResult: AsrResult {
    fn cancellation_reason(&self) -> Result<Result_CancellationReason> {
        let mut n = 0 as Result_CancellationReason;
        hr!(result_get_reason_canceled(self.handle(), &mut n))?;
        Ok(n)
    }

    fn code(&self) -> Result<Result_CancellationErrorCode> {
        let mut n = 0 as Result_CancellationErrorCode;
        hr!(result_get_canceled_error_code(self.handle(), &mut n))?;
        Ok(n)
    }

    fn details(&self) -> Result<String> {
        self.get_by_id(PropertyId_SpeechServiceResponse_JsonErrorDetails)
    }

    fn cancellation_error(&self) -> Result<Recognition> {
        let reason = self.cancellation_reason()?;
        let code = self.code()?;
        let details = self.details()?;
        Err(CancellationError {
            reason,
            code,
            details,
        }
        .into())
    }
}

pub trait NoMatchResult: AsrResult {
    fn no_match_reason(&self) -> Result<Result_NoMatchReason> {
        let mut n = 0 as Result_NoMatchReason;
        hr!(result_get_no_match_reason(self.handle(), &mut n))?;
        Ok(n)
    }

    fn no_match_error(&self) -> Result<Recognition> {
        let reason = self.no_match_reason()?;
        Err(NoMatchError { reason }.into())
    }
}
