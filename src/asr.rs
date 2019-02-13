use crate::speech_api::{
    speech_config_from_subscription, speech_config_get_property_bag,
    speech_config_release, PropertyId_SpeechServiceAuthorization_Token,
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
};
use crate::{hr, properities::Properties, Result};

use std::{ffi::CString, ptr::null_mut};

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
        let handle = null_mut();
        let subscription = CString::new(subscription)?;
        let region = CString::new(region)?;
        unsafe {
            hr(speech_config_from_subscription(
                handle,
                subscription.as_ptr(),
                region.as_ptr(),
            ))?;
            let h_props = null_mut();
            hr(speech_config_get_property_bag(*handle, h_props))?;
            let props = Properties::new(*h_props);
            Ok(RecognizerConfig {
                handle: *handle,
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
}

impl Drop for RecognizerConfig {
    fn drop(&mut self) {
        unsafe { speech_config_release(self.handle) };
    }
}
