use super::{
    audio::AudioInput,
    events::Flags,
    recognizer::{IntentTrigger, Model, Recognizer},
};
use crate::{
    hr,
    properties::{Properties, PropertyBag},
    speech_api::*,
    DeriveHandle, FlattenProps, Handle, Result, INVALID_HANDLE,
};
use serde::{Deserialize, Serialize};
use std::ffi::CString;

macro_rules! DefineAttribute {
    ($name:ident, $setter:ident, $t:ty) => (
        pub fn $name(&self) -> &$t {
            &self.$name
        }
        pub fn $setter<T: Into<$t>>(&mut self, v: T) -> &mut Self {
            self.$name = v.into();
            self
        }
    )
}

macro_rules! SimpleAttribute {
    ($name:ident, $setter:ident, $t:ty) => (
        pub fn $name(&self) -> &$t {
            &self.$name
        }
        pub fn $setter(&mut self, v: $t) -> &mut Self {
            self.$name = v;
            self
        }
    )
}

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
    model_id: String,
    intents: Vec<String>,
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
            model_id: String::new(),
            intents: Vec::new(),
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
            self.flags | Flags::Speech,
            self.timeout,
        ))
    }

    pub fn intent_recognizer(&self) -> Result<Recognizer> {
        let mut audio = self.audio_input()?;
        let mut rh = INVALID_HANDLE;
        hr!(recognizer_create_intent_recognizer_from_config(
            &mut rh,
            self.handle,
            audio.handle(),
        ))?;

        let reco = Recognizer::new(
            rh,
            audio.take_stream(),
            self.flags | Flags::Intent,
            self.timeout,
        );
        self.apply_intents(&reco)?;
        Ok(reco)
    }

    fn apply_intents(&self, reco: &Recognizer) -> Result {
        if self.model_id.is_empty() {
            for ref phrase in &self.intents {
                let trigger = IntentTrigger::from_phrase(phrase)?;
                reco.add_intent(phrase, &trigger)?;
            }
            return Ok(());
        }

        let model = Model::from_app_id(&self.model_id)?;
        if self.intents.is_empty() {
            let trigger = IntentTrigger::from_model_all(&model)?;
            return reco.add_intent("", &trigger);
        }

        for ref intent in &self.intents {
            let trigger = IntentTrigger::from_model(&model, intent)?;
            reco.add_intent(intent, &trigger)?;
        }

        Ok(())
    }

    pub fn audio_input(&self) -> Result<AudioInput> {
        if self.audio_file_path.is_empty() {
            AudioInput::from_config(&self.audio)
        } else {
            AudioInput::from_wav_file(&self.audio_file_path)
        }
    }

    SimpleAttribute!(flags, set_flags, Flags);
    SimpleAttribute!(timeout, set_timeout, u32);
    DefineAttribute!(audio_file_path, set_audio_file_path, String);
    DefineAttribute!(audio, set_audio, AudioConfig);
    DefineAttribute!(model_id, set_model_id, String);
    DefineAttribute!(intents, set_intents, Vec<String>);

    pub fn add_intent(&mut self, name: &str) -> Result<&mut Self> {
        self.intents.push(name.to_string());
        Ok(self)
    }

    DefineProperty!(
        language,
        put_language,
        PropertyId_SpeechServiceConnection_RecoLanguage
    );

    DefineProperty!(
        detailed_result,
        put_detailed_result,
        PropertyId_SpeechServiceResponse_RequestDetailedResultTrueFalse
    );

    pub fn put_proxy(&mut self, proxy: &ProxyConfig) -> Result<&mut Self> {
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
