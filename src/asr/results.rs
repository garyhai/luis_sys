use crate::{
    ffi_get_string, hr,
    properities::Properties,
    speech_api::{
        recognizer_result_handle_is_valid, recognizer_result_handle_release,
        result_get_duration, result_get_offset, result_get_property_bag,
        result_get_reason, result_get_result_id, result_get_text, PropertyId,
        Result_Reason, SPXRESULTHANDLE,
    },
    DeriveSpxHandle, Handle, Result, SpxHandle,
};
use std::{ptr::null_mut, time::Duration};

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

DeriveSpxHandle!(
    RecognitionResult,
    recognizer_result_handle_release,
    recognizer_result_handle_is_valid
);

pub struct RecognitionResult {
    handle: SPXRESULTHANDLE,
    id: Option<String>,
    reason: Option<Result_Reason>,
    text: Option<String>,
    duration: Option<Duration>,
    offset: Option<Duration>,
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

    pub fn reason(&mut self) -> Result<Result_Reason> {
        if self.reason.is_none() {
            let mut reason = 0 as Result_Reason;
            unsafe {
                hr(result_get_reason(self.handle, &mut reason))?;
            }
            self.reason = Some(reason);
        }
        Ok(self.reason.unwrap())
    }

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
}

impl From<Handle> for RecognitionResult {
    fn from(handle: Handle) -> Self {
        Self::new(handle as SPXRESULTHANDLE)
    }
}
