#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use super::recognizer::RecognizerEvent;
use crate::{
    ffi_get_string, hr,
    properities::Properties,
    speech_api::{
        recognizer_recognition_event_get_result,
        recognizer_result_handle_is_valid, recognizer_result_handle_release,
        result_get_canceled_error_code, result_get_duration, result_get_offset,
        result_get_property_bag, result_get_reason, result_get_reason_canceled,
        result_get_result_id, result_get_text, PropertyId,
        PropertyId_SpeechServiceResponse_JsonErrorDetails,
        Result_CancellationErrorCode, Result_CancellationReason,
        Result_NoMatchReason, Result_Reason, SPXRESULTHANDLE,
    },
    DeriveSpxHandle, Handle, Result, SpxHandle,
};
use serde::Serialize;
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ptr::null_mut,
    time::Duration,
};

macro_rules! create_string_prop {
    ($name:ident, $func:ident) => (
        pub fn $name(&mut self) -> Result<&str> {
            if self.$name.is_none() {
                let v = ffi_get_string!($func, self.handle);
                self.$name = Some(v);
            }
            Ok(self.$name.as_ref().unwrap())
        }
    )
}

macro_rules! create_u32_prop {
    ($name:ident, $func:ident) => (
        pub fn $name(&mut self) -> Result<u32> {
            if self.$name.is_none() {
                let mut n = 0u32;
                unsafe {
                    r#try!(hr($func(self.handle, &mut n)));
                }
                self.$name = Some(n);
            }
            Ok(self.$name.unwrap())
        }
    )
}

macro_rules! create_duration_prop {
    ($name:ident, $func:ident) => (
        pub fn $name(&mut self) -> Result<Duration> {
            if self.$name.is_none() {
                let mut duration = 0u64;
                unsafe {
                    r#try!(hr($func(self.handle, &mut duration)));
                }
                let duration = Duration::from_nanos(duration * 100);
                self.$name = Some(duration);
            }
            Ok(self.$name.unwrap())
        }
    )
}

macro_rules! create_props {
    () => (
        fn props(&mut self) -> Result<&Properties> {
            if self.props.is_none() {
                let mut hprops = null_mut();
                unsafe {
                    hr(result_get_property_bag(self.handle, &mut hprops))?;
                }
                self.props = Some(Properties::new(hprops));
            }
            Ok(self.props.as_ref().unwrap())
        }

        pub fn get_by_id(&mut self, id: PropertyId) -> Result<String> {
            self.props()?.get_by_id(id)
        }

        pub fn get_by_name(&mut self, name: &str) -> Result<String> {
            self.props()?.get_by_name(name)
        }
    )
}

macro_rules! create_from_event {
    () => {
        pub fn from_event(evt: RecognizerEvent) -> Result<Self> {
            let mut handle = null_mut();
            hr(unsafe {
                recognizer_recognition_event_get_result(
                    evt.handle(),
                    &mut handle,
                )
            })?;
            Ok(Self::new(handle))
        }
    };
}

DeriveSpxHandle!(
    RecognitionResult,
    recognizer_result_handle_release,
    recognizer_result_handle_is_valid
);

#[derive(Serialize)]
pub struct RecognitionResult {
    #[serde(skip)]
    handle: SPXRESULTHANDLE,
    id: Option<String>,
    reason: Option<Result_Reason>,
    text: Option<String>,
    duration: Option<Duration>,
    offset: Option<Duration>,
    #[serde(skip)]
    props: Option<Properties>,
}

impl RecognitionResult {
    pub fn new(handle: SPXRESULTHANDLE) -> Self {
        RecognitionResult {
            handle,
            id: None,
            reason: None,
            text: None,
            duration: None,
            offset: None,
            props: None,
        }
    }

    create_string_prop!(id, result_get_result_id);
    create_string_prop!(text, result_get_text);
    create_duration_prop!(duration, result_get_duration);
    create_duration_prop!(offset, result_get_offset);
    create_u32_prop!(reason, result_get_reason);
    create_props!();
    create_from_event!();
}

impl From<Handle> for RecognitionResult {
    fn from(handle: Handle) -> Self {
        Self::new(handle as SPXRESULTHANDLE)
    }
}

DeriveSpxHandle!(
    CancellationResult,
    recognizer_result_handle_release,
    recognizer_result_handle_is_valid
);

#[derive(Serialize)]
pub struct CancellationResult {
    #[serde(skip)]
    handle: SPXRESULTHANDLE,
    id: Option<String>,
    reason: Option<Result_CancellationReason>,
    errorCode: Option<Result_CancellationErrorCode>,
    details: Option<String>,
    #[serde(skip)]
    props: Option<Properties>,
}

impl CancellationResult {
    pub fn new(handle: SPXRESULTHANDLE) -> Self {
        CancellationResult {
            handle,
            id: None,
            reason: None,
            errorCode: None,
            details: None,
            props: None,
        }
    }

    create_string_prop!(id, result_get_result_id);
    create_u32_prop!(reason, result_get_reason_canceled);
    create_u32_prop!(errorCode, result_get_canceled_error_code);
    create_props!();
    create_from_event!();

    pub fn details(&mut self) -> Result<&str> {
        if self.details.is_none() {
            let js = self
                .get_by_id(PropertyId_SpeechServiceResponse_JsonErrorDetails)?;
            self.details = Some(js);
        }
        Ok(self.details.as_ref().unwrap())
    }
}

impl From<Handle> for CancellationResult {
    fn from(handle: Handle) -> Self {
        Self::new(handle as SPXRESULTHANDLE)
    }
}

impl Display for CancellationResult {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let code = self.errorCode().unwrap_or(0);
        write!(f, "ASR cancelled by code {}", code)
    }
}

impl Debug for CancellationResult {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let _code = self.errorCode();
        let _reason = self.reason();
        let _details = self.details();
        let _id = self.id();
        write!(
            f,
            "Cancellation: {{id: {:?}, errorCode: {:?}, reason: {:?},\
             details: {:?}}}",
            self.id, self.errorCode, self.reason, self.details
        )
    }
}

DeriveSpxHandle!(
    NoMatchResult,
    recognizer_result_handle_release,
    recognizer_result_handle_is_valid
);

#[derive(Debug, Serialize)]
pub struct NoMatchResult {
    #[serde(skip)]
    handle: SPXRESULTHANDLE,
    id: Option<String>,
    reason: Option<Result_NoMatchReason>,
    #[serde(skip)]
    props: Option<Properties>,
}

impl NoMatchResult {
    pub fn new(handle: SPXRESULTHANDLE) -> Self {
        NoMatchResult {
            handle,
            id: None,
            reason: None,
            props: None,
        }
    }

    create_string_prop!(id, result_get_result_id);
    create_u32_prop!(reason, result_get_reason_canceled);
    create_props!();
    create_from_event!();
}

impl From<Handle> for NoMatchResult {
    fn from(handle: Handle) -> Self {
        Self::new(handle as SPXRESULTHANDLE)
    }
}

impl Display for NoMatchResult {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let reason = self.reason().unwrap_or(0);
        write!(f, "result was not recognized by reason code {}", reason)
    }
}

#[derive(Serialize)]
pub enum Results {
    recognition(RecognitionResult),
    cancellation(CancellationResult),
    noMatch(NoMatchResult),
    none,
}
