use super::{events::Flags, recognizer::Recognizer};
use crate::{
    audio::AudioInput,
    hr,
    properties::{Properties, PropertyBag},
    speech_api::{
        recognizer_create_speech_recognizer_from_config,
        speech_config_from_authorization_token, speech_config_from_endpoint,
        speech_config_from_subscription, speech_config_get_property_bag,
        speech_config_is_handle_valid, speech_config_release,
        PropertyId_SpeechServiceConnection_ProxyHostName,
        PropertyId_SpeechServiceConnection_ProxyPassword,
        PropertyId_SpeechServiceConnection_ProxyPort,
        PropertyId_SpeechServiceConnection_ProxyUserName,
        PropertyId_SpeechServiceConnection_RecoLanguage,
        PropertyId_SpeechServiceResponse_RequestDetailedResultTrueFalse,
        SPXSPEECHCONFIGHANDLE, UINT32_MAX,
    },
    DeriveHandle, FlattenProps, Handle, Result, INVALID_HANDLE,
};
use serde::{Deserialize, Serialize};
use std::ffi::CString;

macro_rules! DefineProperty {
    ($getter:ident, $setter:ident, $id:expr) => (
        pub fn $getter(&self) -> Result<String> {
            self.get_by_id($id)
        }

        pub fn $setter<T: ToString>(&mut self, v: T) -> Result<&mut Self> {
            self.put_by_id($id, v)?;
            Ok(self)
        }
    )
}

DeriveHandle!(
    RecognizerConfig,
    SPXSPEECHCONFIGHANDLE,
    speech_config_release,
    speech_config_is_handle_valid
);

pub struct RecognizerConfig {
    flags: Flags,
    audio: AudioConfig,
    audio_file_path: String,
    timeout: u32,
    handle: SPXSPEECHCONFIGHANDLE,
    props: Properties,
}

impl RecognizerConfig {
    fn new(handle: SPXSPEECHCONFIGHANDLE) -> Result<Self> {
        let mut hprops = INVALID_HANDLE;
        hr!(speech_config_get_property_bag(handle, &mut hprops))?;
        Ok(RecognizerConfig {
            handle,
            props: Properties::new(hprops),
            flags: Flags::Recognized,
            audio: AudioConfig::default(),
            audio_file_path: String::new(),
            timeout: UINT32_MAX,
        })
    }

    pub fn from_subscription(subscription: &str, region: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let subscription = CString::new(subscription)?;
        let region = CString::new(region)?;
        hr!(speech_config_from_subscription(
            &mut handle,
            subscription.as_ptr(),
            region.as_ptr(),
        ))?;
        RecognizerConfig::new(handle)
    }

    pub fn from_authorization_token(token: &str, region: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let token = CString::new(token)?;
        let region = CString::new(region)?;
        hr!(speech_config_from_authorization_token(
            &mut handle,
            token.as_ptr(),
            region.as_ptr(),
        ))?;
        RecognizerConfig::new(handle)
    }

    pub fn from_endpoint(endpoint: &str, subscription: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let endpoint = CString::new(endpoint)?;
        let subscription = CString::new(subscription)?;
        hr!(speech_config_from_endpoint(
            &mut handle,
            endpoint.as_ptr(),
            subscription.as_ptr(),
        ))?;
        RecognizerConfig::new(handle)
    }

    pub fn recognizer(&self) -> Result<Recognizer> {
        let mut audio = self.audio_input()?;
        let mut rh = INVALID_HANDLE;
        hr!(recognizer_create_speech_recognizer_from_config(
            &mut rh,
            self.handle,
            audio.handle(),
        ))?;
        Ok(Recognizer::new(
            rh,
            audio.take_stream(),
            self.flags,
            self.timeout,
        ))
    }

    pub fn audio_input(&self) -> Result<AudioInput> {
        if self.audio_file_path.is_empty() {
            AudioInput::from_config(&self.audio)
        } else {
            AudioInput::from_wav_file(&self.audio_file_path)
        }
    }

    pub fn flags(&self) -> Flags {
        self.flags
    }
    pub fn set_flags(&mut self, v: Flags) -> Result<&mut Self> {
        self.flags = v;
        Ok(self)
    }

    pub fn timeout(&self) -> u32 {
        self.timeout
    }
    pub fn set_timeout(&mut self, v: u32) -> Result<&mut Self> {
        self.timeout = v;
        Ok(self)
    }

    pub fn audio_file_path(&self) -> Result<&str> {
        Ok(self.audio_file_path.as_str())
    }
    pub fn set_audio_file_path(&mut self, v: &str) -> Result<&mut Self> {
        self.audio_file_path = v.to_string();
        Ok(self)
    }

    pub fn audio(&self) -> Result<AudioConfig> {
        Ok(self.audio)
    }
    pub fn set_audio<T: Into<AudioConfig>>(
        &mut self,
        v: T,
    ) -> Result<&mut Self> {
        self.audio = v.into();
        Ok(self)
    }

    DefineProperty!(
        language,
        set_language,
        PropertyId_SpeechServiceConnection_RecoLanguage
    );

    DefineProperty!(
        detailed_result,
        set_detailed_result,
        PropertyId_SpeechServiceResponse_RequestDetailedResultTrueFalse
    );

    pub fn set_proxy(&mut self, proxy: &ProxyConfig) -> Result<&mut Self> {
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
        )?;

        Ok(self)
    }
}

FlattenProps!(RecognizerConfig);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AudioConfig {
    pub rate: u32,
    pub bits: u8,
    pub channels: u8,
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig {
            rate: 16_000,
            bits: 16,
            channels: 1,
        }
    }
}

impl From<(u32, u8, u8)> for AudioConfig {
    fn from(trio: (u32, u8, u8)) -> Self {
        AudioConfig {
            rate: trio.0,
            bits: trio.1,
            channels: trio.2,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProxyConfig {
    host_name: String,
    port: u32,
    user_name: String,
    password: String,
}
