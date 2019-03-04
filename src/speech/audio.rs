use super::builder::AudioConfig;
use crate::speech_api::*;
use crate::{
    hr, DeriveHandle, Handle, Result, SmartHandle, SpxError, INVALID_HANDLE,
};
use std::ffi::CString;

DeriveHandle!(
    AudioInput,
    SPXAUDIOCONFIGHANDLE,
    audio_config_release,
    audio_config_is_handle_valid
);

pub struct AudioInput {
    handle: SPXAUDIOCONFIGHANDLE,
    stream: Option<AudioStream>,
}

impl AudioInput {
    pub fn into_stream(mut self) -> Result<AudioStream> {
        self.stream.take().ok_or(SpxError::NulError)
    }

    pub fn take_stream(&mut self) -> Option<AudioStream> {
        self.stream.take()
    }

    pub fn from_config(cfg: &AudioConfig) -> Result<Self> {
        let stream = AudioStream::from_config(cfg)?;
        Self::from_stream(stream)
    }

    pub fn from_wav_file(path: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let path = CString::new(path)?;
        hr!(audio_config_create_audio_input_from_wav_file_name(
            &mut handle,
            path.as_ptr(),
        ))?;
        Ok(AudioInput {
            handle,
            stream: None,
        })
    }

    pub fn from_stream(stream: AudioStream) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(audio_config_create_audio_input_from_stream(
            &mut handle,
            stream.handle()
        ))?;
        let stream = Some(stream);
        Ok(AudioInput { handle, stream })
    }
}

SmartHandle!(
    AudioStream,
    SPXAUDIOSTREAMHANDLE,
    audio_stream_release,
    audio_stream_is_handle_valid
);

impl AudioStream {
    pub fn from_format(af: &AudioStreamFormat) -> Result<Self> {
        let mut hstream = INVALID_HANDLE;
        hr!(audio_stream_create_push_audio_input_stream(
            &mut hstream,
            af.handle()
        ))?;
        Ok(AudioStream::new(hstream))
    }
    pub fn from_config(cfg: &AudioConfig) -> Result<Self> {
        let af = AudioStreamFormat::from_config(cfg)?;
        Self::from_format(&af)
    }

    pub fn write(&self, buffer: &mut [u8]) -> Result {
        let buf = buffer.as_mut_ptr();
        let size = buffer.len();
        hr!(push_audio_input_stream_write(self.handle, buf, size as u32))
    }

    pub fn close(&self) -> Result {
        hr!(push_audio_input_stream_close(self.handle))
    }
}

SmartHandle!(
    AudioStreamFormat,
    SPXAUDIOSTREAMFORMATHANDLE,
    audio_stream_format_release,
    audio_stream_format_is_handle_valid
);

impl AudioStreamFormat {
    pub fn from_config(cfg: &AudioConfig) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(audio_stream_format_create_from_waveformat_pcm(
            &mut handle,
            cfg.rate,
            cfg.bits,
            cfg.channels
        ))?;
        Ok(AudioStreamFormat::new(handle))
    }

    pub fn from_default() -> Self {
        let mut handle = INVALID_HANDLE;
        unsafe {
            audio_stream_format_create_from_default_input(&mut handle);
        }
        AudioStreamFormat { handle }
    }
}
