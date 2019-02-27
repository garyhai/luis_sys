use crate::speech_api::{
    audio_config_create_audio_input_from_wav_file_name,
    audio_config_is_handle_valid, audio_config_release, SPXAUDIOCONFIGHANDLE,
};
use crate::{hr, Result, SmartHandle, INVALID_HANDLE};

use std::ffi::CString;

SmartHandle!(
    AudioInput,
    SPXAUDIOCONFIGHANDLE,
    audio_config_release,
    audio_config_is_handle_valid
);

impl AudioInput {
    pub fn from_wav_file(path: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let path = CString::new(path)?;
        hr!(audio_config_create_audio_input_from_wav_file_name(
            &mut handle,
            path.as_ptr(),
        ))?;
        Ok(AudioInput::new(handle))
    }
}
