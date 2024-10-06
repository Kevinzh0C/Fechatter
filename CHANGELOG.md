# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added
- Added user status validation during token refresh

### Changed
- Refactored middleware and auth components to reduce coupling
- Improved token refresh security and performance
- Made password verification non-blocking

### Fixed
- Fixed race condition in token refresh (#RT-01)
- Fixed blocking password hash check (#RT-02)
- Fixed disabled users obtaining refresh tokens (#USR-07)
- Consolidated duplicate JWT parsing helpers (#MISC-9)
