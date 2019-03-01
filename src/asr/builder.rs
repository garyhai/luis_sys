use super::{events::Flags, recognizer::Recognizer};
use crate::{
    audio::{AudioConfig, AudioInput},
    hr,
    properities::{Properties, PropertyBag},
    speech_api::{
        recognizer_create_speech_recognizer_from_config,
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
        SPXSPEECHCONFIGHANDLE, UINT32_MAX,
    },
    DeriveHandle, Handle, Result, INVALID_HANDLE,
};
use std::ffi::CString;

macro_rules! create_prop {
    ($name:ident) => (
        pub fn $name<T: ToString>(mut self, v: T) -> Self {
            self.$name = v.to_string();
            self
        }
    );

    ($name:ident, $t:ty) => (
        pub fn $name(mut self, v: $t) -> Self {
            self.$name = v;
            self
        }
    );

    ($prop_get:ident, $prop_put:ident, $id:expr) => (
        pub fn $prop_get(&self) -> Result<String> {
            self.props.get_by_id($id)
        }

        pub fn $prop_put(&self, v: &str) -> Result {
            self.props.put_by_id($id, v)
        }
    )
}

pub struct ProxyConfig {
    host_name: String,
    port: u32,
    user_name: String,
    password: String,
}

DeriveHandle!(
    RecognizerConfig,
    SPXSPEECHCONFIGHANDLE,
    speech_config_release,
    speech_config_is_handle_valid
);

pub struct RecognizerConfig {
    handle: SPXSPEECHCONFIGHANDLE,
    props: Properties,
}

impl RecognizerConfig {
    pub fn new(handle: SPXSPEECHCONFIGHANDLE) -> Result<Self> {
        let mut hprops = INVALID_HANDLE;
        hr!(speech_config_get_property_bag(handle, &mut hprops))?;
        let props = Properties::new(hprops);
        Ok(RecognizerConfig { handle, props })
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

    create_prop!(
        language,
        set_language,
        PropertyId_SpeechServiceConnection_RecoLanguage
    );

    create_prop!(
        endpoint_id,
        set_endpoint_id,
        PropertyId_SpeechServiceConnection_EndpointId
    );

    create_prop!(
        authorization_token,
        set_authorization_token,
        PropertyId_SpeechServiceAuthorization_Token
    );

    create_prop!(
        subscription_key,
        set_subscription_key,
        PropertyId_SpeechServiceConnection_Key
    );

    create_prop!(
        region,
        set_region,
        PropertyId_SpeechServiceConnection_Region
    );

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
}

impl PropertyBag for RecognizerConfig {
    fn get_by_id(&self, id: PropertyId) -> Result<String> {
        self.props.get_by_id(id)
    }

    fn put_by_id(&self, id: PropertyId, value: &str) -> Result<()> {
        self.props.put_by_id(id, value)
    }
}

#[derive(Default)]
pub struct Builder {
    flags: Flags,
    language: String,
    region: String,
    subscription_key: String,
    timeout: u32,
    audio_file_path: String,
    audio: AudioConfig,
}

impl Builder {
    pub fn new() -> Self {
        let flags = Flags::Recognized;
        let timeout = UINT32_MAX;
        Builder {
            flags,
            timeout,
            ..Default::default()
        }
    }

    pub fn build(&self) -> Result<Recognizer> {
        let config = self.create_config()?;
        let mut audio = self.create_audio()?;
        let mut rh = INVALID_HANDLE;
        hr!(recognizer_create_speech_recognizer_from_config(
            &mut rh,
            config.handle(),
            audio.handle(),
        ))?;
        Ok(Recognizer::new(
            rh,
            audio.take_stream(),
            self.flags,
            self.timeout,
        ))
    }

    pub fn create_config(&self) -> Result<RecognizerConfig> {
        let config = RecognizerConfig::from_subscription(
            &self.subscription_key,
            &self.region,
        )?;
        config.set_language(&self.language)?;
        Ok(config)
    }

    pub fn create_audio(&self) -> Result<AudioInput> {
        if self.audio_file_path.is_empty() {
            AudioInput::from_config(&self.audio)
        } else {
            AudioInput::from_wav_file(&self.audio_file_path)
        }
    }

    create_prop!(subscription_key);
    create_prop!(region);
    create_prop!(language);
    create_prop!(audio_file_path);
    create_prop!(audio, AudioConfig);
    create_prop!(flags, Flags);
    create_prop!(timeout, u32);
}
