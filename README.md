# privatemail

[![Rust](https://github.com/ch3ck/privatemail/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/ch3ck/privatemail/actions/workflows/rust.yml)
[![Terraform](https://github.com/ch3ck/privatemail/actions/workflows/terraform.yml/badge.svg?branch=master)](https://github.com/ch3ck/privatemail/actions/workflows/terraform.yml)

[![forthebadge](https://forthebadge.com/images/badges/powered-by-black-magic.svg)](https://github.com/ch3ck/privatemail)
[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://github.com/ch3ck/privatemail)


Forward emails from verified domains on SES to `verified email`.


## Description

A RUST service for AWS Lambda that uses the inbound/outbound capabilities of AWS SES to run a serverless email forwarding service.

For example, if an email is sent from `john@doe.example` to `achu@fufu.soup` is forwarded to a verified email `onions@suya.io`.
This service processes the `From` and `Reply-To` headers set as follows:
```
From: John Doe at john@doe.example <achu@fufu.soup>
Reply-To: john@doe.example

<html>Email body</html>

received by -- <onions@suya.io>
```

### Pre-requisites
- [Verify SES Domain on SES](http://docs.aws.amazon.com/ses/latest/DeveloperGuide/verify-domains.html)
- [Verify SES Email forwarding](https://docs.aws.amazon.com/ses/latest/DeveloperGuide/verify-email-addresses.html)
- [Terraform binaries](https://learn.hashicorp.com/tutorials/terraform/install-cli)
- [AWS SES Limits](https://docs.aws.amazon.com/ses/latest/DeveloperGuide/limits.html)
- [Generate Terraform cloud tokens](https://www.terraform.io/docs/cloud/users-teams-organizations/users.html#api-tokens)


## Build
```bash
$ cargo build
$ cargo test
```

## Contributing
We would appreciate your contributions, all PRs are wellcome. Please see [CONTRIBUTING.md]() for more information.


## Deployment :rocket:

### Local Deploy
```bash
$ bash release.sh
```

### CI Deployment

Set up the necessary keys on actions
```bash
FROM_EMAIL
TF_API_TOKEN
TO_EMAIL
```
All deployments occur via GitHub Actions.


## License
The scripts and documentation in this project are released under the [MIT License](LICENSE.md)


## Author
- [Nyah Check](https://nyah.dev)

