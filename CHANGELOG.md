# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]


## [Released]

## [0.2.1] - 2022-06-24

### Security
- Add terraform security scans to CI.

### Changed
- Use aws_s3_bucket_versioning module for versioning objects
- Update Cargo packages to latest versions


## [0.2.0] - 2022-03-21

### Security

- Fix security vulnerabilities in dependency tree
- Fix clippy warnings in configs library crate
- Create scorecards yml
- Create security policy MD
- Add terraform package ecosystem to dependabot yml
- Update dependencies based on dependabot #9, #8, #10, #11, #12, #13, #14, #15, #16

### Changed
* Replace handler_fn for servce_fn from latest lambda_runtime crate
* Fix handler function by @ch3ck in https://github.com/ch3ck/privatemail/pull/1
* Using structs to ensure sns message is accessed correctly by @noxasaxon in https://github.com/ch3ck/privatemail/pull/3
* Remove unnecessary policy to allow sendRawEmail by @elysiumn in https://github.com/ch3ck/privatemail/pull/4
* Bump serde from 1.0.132 to 1.0.136 by @dependabot in https://github.com/ch3ck/privatemail/pull/9
* Bump actions/checkout from 2 to 3 by @dependabot in https://github.com/ch3ck/privatemail/pull/8
* Bump tokio from 1.15.0 to 1.16.1 by @dependabot in https://github.com/ch3ck/privatemail/pull/10
* Bump ossf/scorecard-action from 0.0.1 to 1.0.4 by @dependabot in https://github.com/ch3ck/privatemail/pull/11
* Bump lambda_runtime from 0.4.1 to 0.5.0 by @dependabot in https://github.com/ch3ck/privatemail/pull/13
* Bump actions/upload-artifact from 2.3.1 to 3 by @dependabot in https://github.com/ch3ck/privatemail/pull/12
* Bump github/codeql-action from 1.0.26 to 1.1.5 by @dependabot in https://github.com/ch3ck/privatemail/pull/14
* Bump serde_json from 1.0.73 to 1.0.79 by @dependabot in https://github.com/ch3ck/privatemail/pull/15
* Bump tracing from 0.1.29 to 0.1.32 by @dependabot in https://github.com/ch3ck/privatemail/pull/16

## New Contributors
* @dependabot made their first contribution in https://github.com/ch3ck/privatemail/pull/9

**Full Changelog**: https://github.com/ch3ck/privatemail/commits/v0.2.0


## [0.1.0] - 2021-27-01

### Added

- `terraform` provisioning scripts for SES forwarding and creating lambda binary.
- `privatemail` crate for forwarding emails from verified domain to `verified email`.
- GitHub actions workflows for building, testing and deploying code.
- Code contribution guidelines.
- Unit and integration tests for privatemail service.

### Changed
* Fix handler function by @ch3ck in https://github.com/ch3ck/privatemail/pull/1
* Using structs to ensure sns message is accessed correctly by @noxasaxon in https://github.com/ch3ck/privatemail/pull/3
* Remove unnecessary policy to allow sendRawEmail by @elysiumn in https://github.com/ch3ck/privatemail/pull/4

### New Contributors
* @noxasaxon made their first contribution in https://github.com/ch3ck/privatemail/pull/3
* @elysiumn made their first contribution in https://github.com/ch3ck/privatemail/pull/4

**Full Changelog**: https://github.com/ch3ck/privatemail/commits/v0.1.0
