//! Represents specific audio configuration, such as microphone, file, or custom audio streams.
//!

use crate::speech_api::*;
use crate::{
    error, hr, properties::Properties, DeriveHandle, FlattenProps, Handle,
    Result, SmartHandle, INVALID_HANDLE, NULL_HANDLE,
};
use serde::{Deserialize, Serialize};
use std::{
    ffi::CString,
    io::{Cursor, Read, Write},
    os::raw::{c_char, c_int, c_void},
    slice::{from_raw_parts, from_raw_parts_mut},
    sync::{
        mpsc::{channel, Receiver, Sender, TryRecvError},
        Arc, Weak,
    },
};

/// Creates an audio stream format object with the specified PCM waveformat characteristics.
/// Currently, only WAV / PCM with 16-bit samples, 16 kHz sample rate, and a single channel (Mono) is supported.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AudioSpec {
    pub rate: u32,
    pub bits: u8,
    pub channels: u8,
}

impl Default for AudioSpec {
    fn default() -> Self {
        AudioSpec {
            rate: 16_000,
            bits: 16,
            channels: 1,
        }
    }
}

impl From<(u32, u8, u8)> for AudioSpec {
    fn from(trio: (u32, u8, u8)) -> Self {
        AudioSpec {
            rate: trio.0,
            bits: trio.1,
            channels: trio.2,
        }
    }
}

DeriveHandle!(
    Audio,
    SPXAUDIOCONFIGHANDLE,
    audio_config_release,
    audio_config_is_handle_valid
);

/// Audio input mode configuration.
pub struct Audio {
    /// Underlying handle.
    handle: SPXAUDIOCONFIGHANDLE,
    /// Placeholder of audio stream.
    stream: Option<Box<dyn AudioStream>>,
    /// Internal properties bag.
    props: Properties,
}

impl Audio {
    fn new(handle: SPXAUDIOCONFIGHANDLE) -> Result<Self> {
        if handle == NULL_HANDLE {
            return Ok(Audio {
                handle,
                props: Properties::new(INVALID_HANDLE),
                stream: None,
            });
        }
        let mut hprops = INVALID_HANDLE;
        hr!(audio_config_get_property_bag(handle, &mut hprops))?;
        Ok(Audio {
            handle,
            props: Properties::new(hprops),
            stream: None,
        })
    }

    /// Create push mode audio input stream.
    pub fn create_push_input(cfg: &AudioSpec) -> Result<Self> {
        let stream = PushAudioInputStream::from_config(cfg)?;
        Self::create_inpput_from_stream(Box::new(stream))
    }

    /// Create pull mode audio input stream.
    pub fn create_pull_input(cfg: &AudioSpec) -> Result<Self> {
        let stream = PullAudioInputStream::from_config(cfg)?;
        Self::create_inpput_from_stream(Box::new(stream))
    }

    /// Create audio output stream.
    pub fn create_output(cfg: &AudioSpec) -> Result<Self> {
        let stream = AudioOutputStream::from_config(cfg)?;
        Self::create_output_from_stream(Box::new(stream))
    }

    /// Create push mode audio input stream.
    pub fn create_push_output(cfg: &AudioSpec) -> Result<Self> {
        let stream = PushAudioOutputStream::from_config(cfg)?;
        Self::create_output_from_stream(Box::new(stream))
    }

    /// Create pull mode audio input stream.
    pub fn create_pull_output(cfg: &AudioSpec) -> Result<Self> {
        let stream = PullAudioOutputStream::from_config(cfg)?;
        Self::create_output_from_stream(Box::new(stream))
    }

    /// Create audio input from wav file. The audio format read from wav file header.
    pub fn create_input_from_wav_file(path: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        let path = CString::new(path)?;
        hr!(audio_config_create_audio_input_from_wav_file_name(
            &mut handle,
            path.as_ptr(),
        ))?;
        Audio::new(handle)
    }

    /// Convert AudioInputStream to AudioInput. AudioInputStream is kept in AudioInput instance.
    pub fn create_inpput_from_stream(
        stream: Box<dyn AudioStream>,
    ) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(audio_config_create_audio_input_from_stream(
            &mut handle,
            stream.handle()
        ))?;
        let mut audio = Audio::new(handle)?;
        audio.stream = Some(stream);
        Ok(audio)
    }

    /// Create audio input from host microphone.
    pub fn create_input_from_microphone() -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(audio_config_create_audio_input_from_default_microphone(
            &mut handle
        ))?;
        Audio::new(handle)
    }

    /// Create audio output to host speaker.
    pub fn create_output_to_speaker() -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(audio_config_create_audio_output_from_default_speaker(
            &mut handle
        ))?;
        Audio::new(handle)
    }

    /// Create audio input from host microphone.
    pub fn create_output_to_file(path: &str) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(audio_config_create_audio_output_from_wav_file_name(
            &mut handle,
            path.as_ptr() as *const c_char,
        ))?;
        Audio::new(handle)
    }

    /// Convert audio output stream to Audio.
    pub fn create_output_from_stream(
        stream: Box<dyn AudioStream>,
    ) -> Result<Self> {
        let mut handle = INVALID_HANDLE;
        hr!(audio_config_create_audio_output_from_stream(
            &mut handle,
            stream.handle()
        ))?;
        let mut audio = Audio::new(handle)?;
        audio.stream = Some(stream);
        Ok(audio)
    }
}

FlattenProps!(Audio);

impl AudioStream for Audio {
    /// Input audio data via created stream.
    fn write(&mut self, buffer: &mut [u8]) -> Result {
        if let Some(stream) = &mut self.stream {
            stream.write(buffer)
        } else {
            Err(error::IsNothing)
        }
    }

    /// Output audio data via created stream.
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        if let Some(stream) = &mut self.stream {
            stream.read(buffer)
        } else {
            Err(error::IsNothing)
        }
    }

    /// Close stream and release resource.
    fn close(&mut self) -> Result {
        if let Some(stream) = &mut self.stream {
            stream.close()
        } else {
            Ok(())
        }
    }
}

pub trait AudioStream: Handle {
    /// The main method to stream audio data.
    fn write(&mut self, _buffer: &mut [u8]) -> Result {
        Err(error::Unimplemented)
    }

    /// The main method to stream audio data.
    fn read(&mut self, _buffer: &mut [u8]) -> Result<usize> {
        Err(error::Unimplemented)
    }

    /// Close the stream gracefully.
    fn close(&mut self) -> Result {
        Ok(())
    }
}

// pub struct StreamReader {
//     receiver: Receiver<Vec<u8>>,
//     buffer: Option<Cursor<Vec<u8>>>,
//     closing: bool,
// }

// impl StreamReader {
//     pub fn new(receiver: Receiver<Vec<u8>>) -> Self {
//         StreamReader {
//             receiver,
//             buffer: None,
//             closing: false,
//         }
//     }
// }

// impl Read for StreamReader {
//     fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
//         if self.closing {
//             return Ok(0);
//         }

//         let buf_size = buffer.len();
//         let mut read_size = 0;
//         if let Some(cache) = &mut self.buffer {
//             read_size = cache.read(buffer)?;
//             if read_size < buf_size {
//                 self.buffer = None;
//             } else {
//                 return Ok(read_size);
//             }
//         }

//         while buf_size > read_size {
//             match self.receiver.try_recv() {
//                 Ok(data) => {
//                     let data_size = data.len();
//                     if data_size == 0 {
//                         self.closing = true;
//                         break;
//                     }
//                     let mut cache = Cursor::new(data);
//                     let sz = cache.read(&mut buffer[read_size..])?;
//                     read_size = read_size + sz;
//                     if sz < data_size {
//                         self.buffer = Some(cache);
//                         break;
//                     }
//                 }
//                 Err(TryRecvError::Empty) => (),
//                 Err(TryRecvError::Disconnected) => {
//                     return Err(IoError::from(ErrorKind::ConnectionAborted))
//                 }
//             }
//         }

//         if read_size > 0 {
//             return Ok(read_size);
//         }

//         let cache = self
//             .receiver
//             .recv()
//             .map_err(|err| IoError::new(ErrorKind::BrokenPipe, err))?;
//         if cache.len() == 0 {
//             self.closing = true;
//             return Ok(0);
//         } else {
//             self.buffer = Some(Cursor::new(cache));
//             return self.read(buffer);
//         }
//     }
// }

DeriveHandle!(
    PullAudioInputStream,
    SPXAUDIOSTREAMHANDLE,
    audio_stream_release,
    audio_stream_is_handle_valid
);

/// Pull audio input stream.
pub struct PullAudioInputStream {
    handle: SPXAUDIOSTREAMHANDLE,
    writer: Sender<Vec<u8>>,
    _reader: Arc<Receiver<Vec<u8>>>,
}

impl PullAudioInputStream {
    /// Create push stream according to the format.
    pub fn from_config(cfg: &AudioSpec) -> Result<Self> {
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

impl AudioStream for PullAudioInputStream {
    fn write(&mut self, buffer: &mut [u8]) -> Result {
        let buf = buffer.to_owned();
        Ok(self.writer.send(buf)?)
    }

    /// Close the stream gracefully.
    fn close(&mut self) -> Result {
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
    pub fn from_config(cfg: &AudioSpec) -> Result<Self> {
        let af = AudioStreamFormat::from_config(cfg)?;
        let mut hstream = INVALID_HANDLE;
        hr!(audio_stream_create_push_audio_input_stream(
            &mut hstream,
            af.handle()
        ))?;
        Ok(PushAudioInputStream::new(hstream))
    }
}

impl AudioStream for PushAudioInputStream {
    fn write(&mut self, buffer: &mut [u8]) -> Result {
        let buf = buffer.as_mut_ptr();
        let size = buffer.len();
        hr!(push_audio_input_stream_write(self.handle, buf, size as u32))
    }

    /// Close the stream gracefully.
    fn close(&mut self) -> Result {
        hr!(push_audio_input_stream_close(self.handle()))
    }
}

DeriveHandle!(
    PushAudioOutputStream,
    SPXAUDIOSTREAMHANDLE,
    audio_stream_release,
    audio_stream_is_handle_valid
);

/// Push output stream.
pub struct PushAudioOutputStream {
    handle: SPXAUDIOSTREAMHANDLE,
    _writer: Arc<Sender<Vec<u8>>>,
    reader: Receiver<Vec<u8>>,
    buffer: Option<Cursor<Vec<u8>>>,
}

impl PushAudioOutputStream {
    /// Create push stream according to the format.
    pub fn from_config(cfg: &AudioSpec) -> Result<Self> {
        let af = AudioStreamFormat::from_config(cfg)?;
        let mut hstream = INVALID_HANDLE;
        hr!(audio_stream_create_push_audio_output_stream(
            &mut hstream,
            af.handle()
        ))?;
        let (writer, reader) = channel();
        let _writer = Arc::new(writer);
        let w = Box::new(Arc::downgrade(&_writer));
        let context = Box::into_raw(w) as *mut c_void;
        hr!(push_audio_output_stream_set_callbacks(
            hstream,
            context,
            Some(on_stream_write),
            Some(on_output_stream_close)
        ))?;
        Ok(PushAudioOutputStream {
            handle: hstream,
            _writer,
            reader,
            buffer: None,
        })
    }
}

impl AudioStream for PushAudioOutputStream {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let buf_size = buffer.len();
        let mut read_size = 0;
        if let Some(cache) = &mut self.buffer {
            read_size = cache.read(buffer)?;
            if read_size < buf_size {
                self.buffer = None;
            } else {
                return Ok(read_size);
            }
        }
        while buf_size > read_size {
            match self.reader.try_recv() {
                Ok(data) => {
                    let data_size = data.len();
                    if data_size == 0 {
                        break;
                    }
                    let mut cache = Cursor::new(data);
                    let sz = cache.read(&mut buffer[read_size..])?;
                    read_size = read_size + sz;
                    if sz < data_size {
                        self.buffer = Some(cache);
                        break;
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    return Err(error::Other("disconnected".to_string()));
                }
            }
        }

        Ok(read_size)
    }
}

unsafe extern "C" fn on_output_stream_close(context: *mut c_void) {
    log::debug!("Output stream close event fired.");
    if !context.is_null() {
        // Auto release the Box and weak pointer.
        Box::from_raw(context as *mut Weak<Sender<Vec<u8>>>);
    }
}

unsafe extern "C" fn on_stream_write(
    context: *mut c_void,
    buffer: *mut u8,
    size: u32,
) -> c_int {
    if context.is_null() {
        log::error!("Unknown context with NULL pointer when write stream.");
        return 0;
    }
    let ctx = Box::from_raw(context as *mut Weak<Sender<Vec<u8>>>);
    let ctx = Box::leak(ctx); // avoid auto release.
    if let Some(s) = ctx.upgrade() {
        let buf = from_raw_parts(buffer, size as usize);
        match s.send(buf.to_owned()) {
            Ok(()) => return size as c_int,
            Err(err) => log::error!("Audio input stream read error: {}", err),
        }
    }
    log::error!("Cannot get stream reader!");
    return 0;
}

SmartHandle!(
    PullAudioOutputStream,
    SPXAUDIOSTREAMHANDLE,
    audio_stream_release,
    audio_stream_is_handle_valid
);

impl PullAudioOutputStream {
    /// Create push stream according to the format.
    pub fn from_config(cfg: &AudioSpec) -> Result<Self> {
        let af = AudioStreamFormat::from_config(cfg)?;
        let mut hstream = INVALID_HANDLE;
        hr!(audio_stream_create_pull_audio_output_stream(
            &mut hstream,
            af.handle()
        ))?;
        Ok(PullAudioOutputStream::new(hstream))
    }
}

impl AudioStream for PullAudioOutputStream {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let buf = buffer.as_mut_ptr();
        let size = buffer.len();
        let mut filled = 0u32;
        let ptr_filled = &mut filled as *mut u32;
        hr!(pull_audio_output_stream_read(
            self.handle,
            buf,
            size as u32,
            ptr_filled
        ))?;
        Ok(filled as usize)
    }
}

SmartHandle!(
    AudioOutputStream,
    SPXAUDIOSTREAMHANDLE,
    audio_stream_release,
    audio_stream_is_handle_valid
);

impl AudioOutputStream {
    /// Create push stream according to the format.
    pub fn from_config(cfg: &AudioSpec) -> Result<Self> {
        let af = AudioStreamFormat::from_config(cfg)?;
        let mut hstream = INVALID_HANDLE;
        hr!(audio_stream_create_pull_audio_output_stream(
            &mut hstream,
            af.handle()
        ))?;
        Ok(AudioOutputStream::new(hstream))
    }
}

impl AudioStream for AudioOutputStream {}

SmartHandle!(
    AudioStreamFormat,
    SPXAUDIOSTREAMFORMATHANDLE,
    audio_stream_format_release,
    audio_stream_format_is_handle_valid
);

impl AudioStreamFormat {
    /// Create by specs.
    pub fn from_config(cfg: &AudioSpec) -> Result<Self> {
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
