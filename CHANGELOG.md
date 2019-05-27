# CHANGELOG

## [Unreleased]

## [0.3.18] - 2019-5-27

### Added

- Add `update` command to Makefile for upgrading of dependencies.
- Add `get_sdk` command to Makefile to fetch latest version of SpeechSDK for running of `asr_simple`.
- Add more conversions for std error types to `SpxError`
- Add `pull mode` of streaming.

## [0.3.17] - 2019-5-20

### Changed

- Fetch & extract `SpeechSDK` in `build.rs`.

## [0.3.16] - 2019-5-20

### Changed

- Remove SpeechSDK, download that on demand.

## [0.3.15] - 2019-5-16

### Fixed

- Exclude `asr_simple.rs` for leak of keys.
- Exclude `SpeechSDK` to reduce size of crate.

### Added

- Support microphone input of audio stream.

## [0.3.14] - 2019-5-7

### Changed

- Upgrade Microsoft Speech SDK to v1.5.0.
- Add macOS support.

## [0.3.13] - 2019-4-30

### Added

- Add `endpoint` property in `RecognizerConfig` for customized model.


## [0.3.12] - 2019-4-10

### Fixed

- Fix copy/paste bug of `RecognitionResult::offset`.

## [0.3.11] - 2019-3-15

### Changed

- `Recognizer::recognize` return full `EventResult` now.

## [0.3.10] - 2019-3-14

### Added

- Add section for translation and synthesis in `README.md`.

## [0.3.9] - 2019-3-13

### Added

- Translation and translator synthesizing audio.
- More comments copied from Microsoft Speech SDK.

### Changed

- Event of `Canceled` cause stop of recognition. Events `SessionStopped` and `Disconnected` will not be triggered.

## [0.3.8] - 2019-3-8

### Changed

- Increase the version number for International Women's Day 2019.
- Fix spell and date error.

## [0.3.7] - 2019-3-8

### Changed

- Add more content to README.md file.
- Suppress error result of `RecognitionResult::details`.
- Change method name of `EventStream::filter` to `EventStream::set_filter` for name conflic of `Stream::filter`.

## [0.3.6] - 2019-3-6

### Changed

- Convert detailed result of intent recognition to `serde_json::Value` type.
- Make `CancellationResult::cancellation_error` be generic.
- Expose all fields of struct `Recognition` to public.

## [0.3.5] - 2019-3-6

### Changed

- Implicitly add `Flags::Session | Flags::Canceld` for Recognizer startup to avoid unresolved future of EventStream.

## [0.3.4] - 2019-3-5

### Changed

- Re-export sub-modules of speech.
- Get rid of the warning of cargo publish by moving bindings.rs to output directory.

## [0.3.3] - 2019-3-5

### Changed

- Change the crate category to valid **"external-ffi-bindings"**.
- Add properties bag for AudioInput (SpeechSDK v1.3.1).

## [0.3.2] - 2019-3-4

### Changed

- Ignore modification of "asr_simple.rs" for risk of keys leak.
- Re-add "bindings.rs" to lock version in phase of development.
- Add more fields in manifest file.
- Fix errors of inner line doc comments.

## [0.3.1] - 2019-3-4

### Added

- Documentation and manifest for publishing.

## [0.3.0] - 2019-3-4

### Changed

- Change mod name from `asr` to `speech`.

## [0.2.1] - 2019-3-4

### Changed

- Remove `rustc-hash` crate to wait for new version of HashMap.
- Update SpeechSDK to version 1.3.1

## [0.2.0] - 2019-3-4

### Changed

- Makefile `run` command add `LD_LIBRARY_PATH` environment avoid lib version conflication.
- Add intent recognization function.
- Change `reason` of `EventResult` to `Flags` type.
- Change some weird constant types to readable enum types.
- Do not deglob import of speech_api! (match for unimported consts have potential bugs).

## [0.1.0] - 2019-3-1

### Changed

- Add Makefile and CHANGELOG.md
- Fix bug of EventStream filter.
- Rename `EventStream::into_json` to `json`
- Change `EventStream::once` name and behavior.
- Add push stream support.
- Merge Builder into RecognizerConfig.
