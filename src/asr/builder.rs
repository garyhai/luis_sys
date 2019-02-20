use rustc_hash::FxHashMap as Table;
use std::{ffi::CString, ptr::null_mut};

use crate::{
    audio::AudioInput,
    create_prop, hr,
    properities::Properties,
    speech_api::{
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
        SPXSPEECHCONFIGHANDLE,
    },
    DeriveSpxHandle, Handle, Result, SpxHandle,
};
use super::recognizer::Recognizer;

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

    pub fn get_by_id(&self, id: PropertyId) -> Result<String> {
        self.props.get_by_id(id)
    }

    pub fn put_by_id(&self, id: PropertyId, value: &str) -> Result<()> {
        self.props.put_by_id(id, value)
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
