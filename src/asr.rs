use crate::speech_api::{
    recognizer_create_speech_recognizer_from_config,
    recognizer_handle_is_valid, recognizer_handle_release,
    recognizer_recognize_once, recognizer_result_handle_is_valid,
    recognizer_result_handle_release, result_get_duration, result_get_offset,
    result_get_property_bag, result_get_reason, result_get_result_id,
    result_get_text, speech_config_from_subscription,
    speech_config_get_property_bag, speech_config_is_handle_valid,
    speech_config_release, PropertyId,
    PropertyId_SpeechServiceAuthorization_Token,
    PropertyId_SpeechServiceConnection_EndpointId,
    PropertyId_SpeechServiceConnection_Key,
    PropertyId_SpeechServiceConnection_ProxyHostName,
    PropertyId_SpeechServiceConnection_ProxyPassword,
    PropertyId_SpeechServiceConnection_ProxyPort,
    PropertyId_SpeechServiceConnection_ProxyUserName,
    PropertyId_SpeechServiceConnection_RecoLanguage,
    PropertyId_SpeechServiceConnection_Region,
    PropertyId_SpeechServiceResponse_RequestDetailedResultTrueFalse,
    Result_Reason, Result_Reason_ResultReason_RecognizedSpeech, SPXRECOHANDLE,
    SPXRESULTHANDLE, SPXSPEECHCONFIGHANDLE,
};
use crate::{
    audio::AudioInput, hr, properities::Properties, DeriveSpxHandle, Handle,
    Result, SpxError, SpxHandle,
};
use rustc_hash::FxHashMap as Table;
use std::{ffi::CString, ptr::null_mut, time::Duration};

pub struct ProxyConfig {
    host_name: String,
    port: u32,
    user_name: String,
    password: String,
}

DeriveSpxHandle!(
    RecognizerConfig,
    speech_config_release,
    speech_config_is_handle_valid
);

pub struct RecognizerConfig {
    handle: SPXSPEECHCONFIGHANDLE,
    props: Properties,
}

impl RecognizerConfig {
    pub fn from_subscription(subscription: &str, region: &str) -> Result<Self> {
        let mut handle = null_mut();
        let subscription = CString::new(subscription)?;
        let region = CString::new(region)?;
        unsafe {
            hr(speech_config_from_subscription(
                &mut handle,
                subscription.as_ptr(),
                region.as_ptr(),
            ))?;
            let mut hprops = null_mut();
            hr(speech_config_get_property_bag(handle, &mut hprops))?;
            let props = Properties::new(hprops);
            Ok(RecognizerConfig {
                handle: handle,
                props,
            })
        }
    }

    pub fn language(&self) -> Result<String> {
        self.props
            .get_by_id(PropertyId_SpeechServiceConnection_RecoLanguage)
    }

    pub fn set_language(&self, v: &str) -> Result {
        self.props
            .put_by_id(PropertyId_SpeechServiceConnection_RecoLanguage, v)
    }

    pub fn endpoint_id(&self) -> Result<String> {
        self.props
            .get_by_id(PropertyId_SpeechServiceConnection_EndpointId)
    }

    pub fn set_endpoint_id(&self, v: &str) -> Result {
        self.props
            .put_by_id(PropertyId_SpeechServiceConnection_EndpointId, v)
    }

    pub fn authorization_token(&self) -> Result<String> {
        self.props
            .get_by_id(PropertyId_SpeechServiceAuthorization_Token)
    }

    pub fn set_authorization_token(&self, v: &str) -> Result {
        self.props
            .put_by_id(PropertyId_SpeechServiceAuthorization_Token, v)
    }
    pub fn subscription_key(&self) -> Result<String> {
        self.props
            .get_by_id(PropertyId_SpeechServiceAuthorization_Token)
    }

    pub fn set_subscription_key(&self, v: &str) -> Result {
        self.props
            .put_by_id(PropertyId_SpeechServiceConnection_Key, v)
    }
    pub fn region(&self) -> Result<String> {
        self.props
            .get_by_id(PropertyId_SpeechServiceConnection_Region)
    }

    pub fn set_region(&self, v: &str) -> Result {
        self.props
            .put_by_id(PropertyId_SpeechServiceConnection_Region, v)
    }
    pub fn detailed_result(&self) -> Result<bool> {
        let r = self.props.get_by_id(
            PropertyId_SpeechServiceResponse_RequestDetailedResultTrueFalse,
        )?;
        if r == "true" {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn set_detailed_result(&self, v: bool) -> Result {
        let v = if v { "true" } else { "false" };
        self.props.put_by_id(
            PropertyId_SpeechServiceResponse_RequestDetailedResultTrueFalse,
            v,
        )
    }

    pub fn set_proxy(&self, proxy: &ProxyConfig) -> Result {
        self.props.put_by_id(
            PropertyId_SpeechServiceConnection_ProxyHostName,
            &proxy.host_name,
        )?;
        self.props.put_by_id(
            PropertyId_SpeechServiceConnection_ProxyPort,
            &proxy.port.to_string(),
        )?;
        self.props.put_by_id(
            PropertyId_SpeechServiceConnection_ProxyUserName,
            &proxy.user_name,
        )?;
        self.props.put_by_id(
            PropertyId_SpeechServiceConnection_ProxyPassword,
            &proxy.password,
        )
    }

    pub fn get_by_id(&self, id: PropertyId) -> Result<String> {
        self.props.get_by_id(id)
    }

    pub fn put_by_id(&self, id: PropertyId, value: &str) -> Result<()> {
        self.props.put_by_id(id, value)
    }
}

DeriveSpxHandle!(
    Recognizer,
    recognizer_handle_release,
    recognizer_handle_is_valid
);

pub struct Recognizer {
    handle: SPXRECOHANDLE,
    config: RecognizerConfig,
    audio: AudioInput,
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
            })
        }
    }

    pub fn from_subscription(subscription: &str, region: &str) -> Result<Self> {
        let config = RecognizerConfig::from_subscription(subscription, region)?;
        let audio = AudioInput::new();
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
}

pub struct Builder {
    table: Table<PropertyId, String>,
    audio: String,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            table: Table::default(),
            audio: String::new(),
        }
    }

    pub fn subscription_key<T: ToString>(mut self, v: T) -> Self {
        self.table
            .insert(PropertyId_SpeechServiceConnection_Key, v.to_string());
        self
    }

    pub fn region<T: ToString>(mut self, v: T) -> Self {
        self.table
            .insert(PropertyId_SpeechServiceConnection_Region, v.to_string());
        self
    }

    pub fn language<T: ToString>(mut self, v: T) -> Self {
        self.table.insert(
            PropertyId_SpeechServiceConnection_RecoLanguage,
            v.to_string(),
        );
        self
    }

    pub fn audio_file_path<T: ToString>(mut self, v: T) -> Self {
        self.audio = v.to_string();
        self
    }

    pub fn create_config(&self) -> Result<RecognizerConfig> {
        let default = String::from("");
        let key = self
            .table
            .get(&PropertyId_SpeechServiceConnection_Key)
            .unwrap_or(&default);
        let region = self
            .table
            .get(&PropertyId_SpeechServiceConnection_Region)
            .unwrap_or(&default);
        let config = RecognizerConfig::from_subscription(&key, &region)?;
        for (k, v) in self.table.iter() {
            config.put_by_id(*k, v)?;
        }
        Ok(config)
    }

    pub fn create_audio(&self) -> Result<AudioInput> {
        AudioInput::from_wav_file(&self.audio)
    }

    pub fn create_recognizer(&self) -> Result<Recognizer> {
        let config = self.create_config()?;
        let audio = self.create_audio()?;
        Recognizer::from_config(config, audio)
    }
}

macro_rules! ffi_get_string {
    ($f:ident, $h:expr $(, $sz:expr)?) => ({
        let _max_len = 1024;
        $(
            let _max_len = $sz;
        )?
        let s = String::with_capacity(_max_len + 1);
        let buf = r#try!(CString::new(s));
        let buf_ptr = buf.into_raw();
        unsafe {
            r#try!(hr($f($h, buf_ptr, _max_len as u32)));
            let output = CString::from_raw(buf_ptr);
            r#try!(output.into_string())
        }
    })
}

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
