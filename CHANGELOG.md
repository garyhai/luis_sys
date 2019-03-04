# CHANGELOG

## [Unreleased]

### Changed

- Remove auto-generated file "bindings.rs".
- Ignore "bindings.rs" and "asr_simple.rs".

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
