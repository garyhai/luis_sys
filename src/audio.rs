use crate::speech_api::*;
use crate::{hr, Result, SpxHandle, Handle};

use std::{ffi::CString, ptr::null_mut};

pub struct AudioConfig {
    handle: SPXAUDIOCONFIGHANDLE,
}

impl AudioConfig {
    pub fn from_wav_file_input(path: &str) -> Result<Self> {
        let handle = null_mut();
        let path = CString::new(path)?;
        unsafe {
            hr(audio_config_create_audio_input_from_wav_file_name(
                handle,
                path.as_ptr(),
            ))?;
            Ok(AudioConfig { handle: *handle })
        }
    }
}

impl Drop for AudioConfig {
    fn drop(&mut self) {
        unsafe { audio_config_release(self.handle) };
    }
}

impl SpxHandle for AudioConfig {
    fn handle(&self) -> Handle {
        self.handle as Handle
    }
}
