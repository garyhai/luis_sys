# CHANGELOG

## [Unreleased]

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
