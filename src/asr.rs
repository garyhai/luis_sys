use crate::speech_api::{
    recognizer_create_speech_recognizer_from_config,
    recognizer_handle_is_valid, recognizer_handle_release,
    recognizer_recognize_once, result_get_reason, result_get_text,
    speech_config_from_subscription, speech_config_get_property_bag,
    speech_config_is_handle_valid, speech_config_release, PropertyId,
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
    Result_Reason_ResultReason_RecognizedSpeech, SPXRECOHANDLE,
    SPXSPEECHCONFIGHANDLE,
};
use crate::{
    audio::AudioInput, hr, properities::Properties, Handle, Result, SpxError,
    SpxHandle,
};
use rustc_hash::FxHashMap as Table;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    ptr::null_mut,
};

pub struct ProxyConfig {
    host_name: String,
    port: u32,
    user_name: String,
    password: String,
}

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

impl Drop for RecognizerConfig {
    fn drop(&mut self) {
        unsafe {
            if speech_config_is_handle_valid(self.handle) {
                speech_config_release(self.handle);
            }
        }
    }
}

impl SpxHandle for RecognizerConfig {
    fn handle(&self) -> Handle {
        self.handle as Handle
    }
}

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
        let mut rr = 0;
        unsafe {
            hr(recognizer_recognize_once(self.handle, &mut hres))?;
            hr(result_get_reason(hres, &mut rr))?;
            if rr == Result_Reason_ResultReason_RecognizedSpeech {
                let sz = 1024;
                let mut buf: Vec<c_char> = Vec::with_capacity(sz + 1);
                let slice = buf.as_mut_slice();
                let ptr = slice.as_mut_ptr();
                hr(result_get_text(hres, ptr, sz as u32))?;
                let s = CStr::from_ptr(ptr).to_str()?;
                Ok(String::from(s))
            } else {
                Err(SpxError::Unknown)
            }
        }
    }
}

impl Drop for Recognizer {
    fn drop(&mut self) {
        unsafe {
            if recognizer_handle_is_valid(self.handle) {
                recognizer_handle_release(self.handle);
            }
        }
    }
}

impl SpxHandle for Recognizer {
    fn handle(&self) -> Handle {
        self.handle as Handle
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
        let config =
            RecognizerConfig::from_subscription(&key, &region)?;
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
