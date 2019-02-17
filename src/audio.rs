use crate::speech_api::*;
use crate::{hr, Handle, Result, SpxHandle, DeriveSpxHandle};

use std::{ffi::CString, ptr::null_mut};

pub struct AudioInput {
    handle: SPXAUDIOCONFIGHANDLE,
}

impl AudioInput {
    pub fn new() -> Self {
        AudioInput {
            handle: null_mut(),
        }
    }

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

DeriveSpxHandle!(
    AudioInput,
    audio_config_release,
    audio_config_is_handle_valid
);
