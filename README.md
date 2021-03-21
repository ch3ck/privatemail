# privatemail

[![Rust](https://github.com/ch3ck/privatemail/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/ch3ck/privatemail/actions/workflows/rust.yml)
[![Terraform](https://github.com/ch3ck/privatemail/actions/workflows/terraform.yml/badge.svg?branch=master)](https://github.com/ch3ck/privatemail/actions/workflows/terraform.yml)

[![forthebadge](https://forthebadge.com/images/badges/powered-by-black-magic.svg)](https://github.com/ch3ck/privatemail)
[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://github.com/ch3ck/privatemail)




Forward emails from SES to private email server


## Description

A RUST service for AWS Lambda that uses the inbound/outbound capabilities of AWS SES to run a serverless email forwarding service.

For example, if an email is sent from `john@doe.example` to `hello@nyah.dev` is processed by this service with the `From` and `Reply-To` headers set as follows:
```
From: John Doe at john@doe.example <hello@nyah.dev>
Reply-To: john@doe.example
```

This service can be configured to send emails from any verified `fromEmail` to work

### Notes
- http://docs.aws.amazon.com/ses/latest/DeveloperGuide/verify-domains.html
- https://docs.aws.amazon.com/ses/latest/DeveloperGuide/limits.html
- http://docs.aws.amazon.com/ses/latest/DeveloperGuide/limits.html



## Deployment

All deployments occur via Actions


## Author
- [Nyah Check](https://nyah.dev)

