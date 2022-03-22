# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

## [0.2.0] - 2022-03-21

### Changed

- Replace handler_fn for servce_fn from latest lambda_runtime crate

### Security

- Fix security vulnerabilities in dependency tree
- Fix clippy warnings in configs library crate
- Create scorecards yml
- Create security policy MD
- Add terraform package ecosystem to dependabot yml
- Update dependencies based on dependabot #9, #8, #10, #11, #12, #13, #14, #15, #16


## [Released]

## [0.1.0] - 2021-27-01

### Added

- `terraform` provisioning scripts for SES forwarding and creating lambda binary.
- `privemail` crate for forwarding emails from verified domain to `verified email`.
- GitHub actions workflows for building, testing and deploying code.
- Code contribution guidelines.
- Unit and integration tests for privatemail service.
