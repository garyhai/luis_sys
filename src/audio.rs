use crate::speech_api::{
    audio_config_create_audio_input_from_wav_file_name,
    audio_config_is_handle_valid, audio_config_release,
};
use crate::{hr, SmartHandle, Handle, Result, SpxHandle};

use std::{ffi::CString, ptr::null_mut};

SmartHandle!(
    AudioInput,
    audio_config_release,
    audio_config_is_handle_valid
);

impl AudioInput {
    pub fn from_wav_file(path: &str) -> Result<Self> {
        let mut handle = null_mut();
        let path = CString::new(path)?;
        unsafe {
            hr(audio_config_create_audio_input_from_wav_file_name(
                &mut handle,
                path.as_ptr(),
            ))?;
            Ok(AudioInput { handle })
        }
    }
}
