//! Represents specific audio configuration, such as microphone, file, or custom audio streams.
//!

use super::builder::AudioConfig;
use crate::speech_api::*;
use crate::{
    error::IsNothing, hr, properties::Properties, DeriveHandle, FlattenProps,
    Handle, Result, SmartHandle, INVALID_HANDLE, NULL_HANDLE,
};
use std::{
    ffi::CString,
    io::Write,
    os::raw::{c_int, c_void},
    slice::from_raw_parts_mut,
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, Weak,
    },
};

DeriveHandle!(
    AudioInput,
    SPXAUDIOCONFIGHANDLE,
    audio_config_release,
    audio_config_is_handle_valid
);

/// Audio input mode configuration.
pub struct AudioInput {
    /// Underlying handle.
    handle: SPXAUDIOCONFIGHANDLE,
    /// Placeholder of audio stream.
    stream: Option<Box<dyn AudioInputStream>>,
    /// Internal properties bag.
    props: Properties,
}

impl AudioInput {
    fn new(handle: SPXAUDIOCONFIGHANDLE) -> Result<Self> {
        if handle == NULL_HANDLE {
            return Ok(AudioInput {
                handle,
                props: Properties::new(INVALID_HANDLE),
                stream: None,
            });
        }
        let mut hprops = INVALID_HANDLE;
        hr!(audio_config_get_property_bag(handle, &mut hprops))?;
        Ok(AudioInput {
            handle,
            props: Properties::new(hprops),
            stream: None,
        })
    }

    /// Drop AudioInput and yield an audio stream.
    pub fn input(&self, buffer: &mut [u8]) -> Result {
        if let Some(stream) = &self.stream {
            stream.write(buffer)
        } else {
            Err(IsNothing)
        }
    }

    /// Take away audio stream.
    pub fn close(&mut self) -> Result {
        if let Some(stream) = &self.stream {
            stream.close()
        } else {
            Ok(())
        }
    }

    /// Create audio config and stream by given auido format.
    pub fn from_config(cfg: &AudioConfig, pull_mode: bool) -> Result<Self> {
        if pull_mode {
            Self::pull_streaming_from_config(cfg)
        } else {
            Self::push_streaming_from_config(cfg)
        }
    }

    /// Create push mode audio input stream.
    pub fn push_streaming_from_config(cfg: &AudioConfig) -> Result<Self> {
        let stream = PushAudioInputStream::from_config(cfg)?;
        Self::from_stream(Box::new(stream))
    }

    /// Create pull mode audio input stream.
    pub fn pull_streaming_from_config(cfg: &AudioConfig) -> Result<Self> {
        let stream = PullAudioInputStream::from_config(cfg)?;
        Self::from_stream(Box::new(stream))
    }

    /// Create audio input from wav file. The audio format read from wav file header.
    pub fn from_wav_file(path: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let path = CString::new(path)?;
        hr!(audio_config_create_audio_input_from_wav_file_name(
            &mut handle,
            path.as_ptr(),
        ))?;
        AudioInput::new(handle)
    }

    /// Create audio input from host microphone.
    pub fn from_microphone() -> Result<Self> {
        // AudioInput::new(std::ptr::null_mut())
        AudioInput::new(0 as SPXHANDLE)
    }

    /// Convert AudioInputStream to AudioInput. AudioInputStream is kept in AudioInput instance.
    pub fn from_stream(stream: Box<dyn AudioInputStream>) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(audio_config_create_audio_input_from_stream(
            &mut handle,
            stream.handle()
        ))?;
        let mut audio = AudioInput::new(handle)?;
        audio.stream = Some(stream);
        Ok(audio)
    }
}

FlattenProps!(AudioInput);

pub trait AudioInputStream: Handle {
    /// The main method to stream audio data.
    fn write(&self, buffer: &mut [u8]) -> Result;

    /// Close the stream gracefully.
    fn close(&self) -> Result;
}

DeriveHandle!(
    PullAudioInputStream,
    SPXAUDIOSTREAMHANDLE,
    audio_stream_release,
    audio_stream_is_handle_valid
);

/// Pull stream.
pub struct PullAudioInputStream {
    handle: SPXAUDIOSTREAMHANDLE,
    writer: Sender<Vec<u8>>,
    _reader: Arc<Receiver<Vec<u8>>>,
}

impl PullAudioInputStream {
    /// Create push stream according to the format.
    fn from_config(cfg: &AudioConfig) -> Result<Self> {
        let af = AudioStreamFormat::from_config(cfg)?;
        let mut hstream = INVALID_HANDLE;
        hr!(audio_stream_create_pull_audio_input_stream(
            &mut hstream,
            af.handle()
        ))?;
        let (writer, reader) = channel();
        let _reader = Arc::new(reader);
        let r = Box::new(Arc::downgrade(&_reader));
        let context = Box::into_raw(r) as *mut c_void;
        hr!(pull_audio_input_stream_set_callbacks(
            hstream,
            context,
            Some(on_stream_read),
            Some(on_stream_close)
        ))?;
        Ok(PullAudioInputStream {
            handle: hstream,
            writer,
            _reader,
        })
    }
}

impl AudioInputStream for PullAudioInputStream {
    fn write(&self, buffer: &mut [u8]) -> Result {
        let buf = buffer.to_owned();
        Ok(self.writer.send(buf)?)
    }

    /// Close the stream gracefully.
    fn close(&self) -> Result {
        self.write(&mut [])
    }
}

unsafe extern "C" fn on_stream_close(context: *mut c_void) {
    log::debug!("Pull stream close event fired.");
    if !context.is_null() {
        // Auto release the Box and weak pointer.
        Box::from_raw(context as *mut Weak<Receiver<Vec<u8>>>);
    }
}

unsafe extern "C" fn on_stream_read(
    context: *mut c_void,
    buffer: *mut u8,
    size: u32,
) -> c_int {
    if context.is_null() {
        log::error!("Unknown context with NULL pointer when read stream.");
        return 0;
    }
    let ctx = Box::from_raw(context as *mut Weak<Receiver<Vec<u8>>>);
    let ctx = Box::leak(ctx); // avoid auto release.
    if let Some(r) = ctx.upgrade() {
        let mut buf = from_raw_parts_mut(buffer, size as usize);
        let data = match r.try_recv() {
            Ok(data) => data,
            Err(TryRecvError::Empty) => match r.recv() {
                Ok(data) => data,
                Err(err) => {
                    log::error!("Data read error: {}", err);
                    return 0;
                }
            },
            Err(TryRecvError::Disconnected) => {
                log::error!("Data channel is disconnected!");
                return 0;
            }
        };
        let sz = data.len();
        return if sz == 0 {
            0
        } else if sz > buf.len() {
            log::error!(
                "Read buffer size is too small ({} vs {}) ",
                sz,
                buf.len()
            );
            0
        } else {
            buf.write(&data).unwrap()
        } as c_int;
    }
    log::error!("Cannot get stream reader!");
    return 0;
}

SmartHandle!(
    PushAudioInputStream,
    SPXAUDIOSTREAMHANDLE,
    audio_stream_release,
    audio_stream_is_handle_valid
);

impl PushAudioInputStream {
    /// Create push stream according to the format.
    fn from_config(cfg: &AudioConfig) -> Result<Self> {
        let af = AudioStreamFormat::from_config(cfg)?;
        let mut hstream = INVALID_HANDLE;
        hr!(audio_stream_create_push_audio_input_stream(
            &mut hstream,
            af.handle()
        ))?;
        Ok(PushAudioInputStream::new(hstream))
    }
}

impl AudioInputStream for PushAudioInputStream {
    fn write(&self, buffer: &mut [u8]) -> Result {
        let buf = buffer.as_mut_ptr();
        let size = buffer.len();
        hr!(push_audio_input_stream_write(self.handle, buf, size as u32))
    }

    /// Close the stream gracefully.
    fn close(&self) -> Result {
        hr!(push_audio_input_stream_close(self.handle()))
    }
}

SmartHandle!(
    AudioStreamFormat,
    SPXAUDIOSTREAMFORMATHANDLE,
    audio_stream_format_release,
    audio_stream_format_is_handle_valid
);

impl AudioStreamFormat {
    /// Create by specs.
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
    /// Creates a memory backed push stream using the default format (16Khz 16bit mono PCM).
    pub fn from_default() -> Self {
        let mut handle = INVALID_HANDLE;
        unsafe {
            audio_stream_format_create_from_default_input(&mut handle);
        }
        AudioStreamFormat { handle }
    }
}
