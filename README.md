# privatemail

[![Rust](https://github.com/ch3ck/privatemail/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/ch3ck/privatemail/actions/workflows/rust.yml)
[![Terraform](https://github.com/ch3ck/privatemail/actions/workflows/terraform.yml/badge.svg?branch=master)](https://github.com/ch3ck/privatemail/actions/workflows/terraform.yml)

[![forthebadge](https://forthebadge.com/images/badges/powered-by-black-magic.svg)](https://github.com/ch3ck/privatemail)
[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://github.com/ch3ck/privatemail)

Forward emails from verified domains on SES to SES `verified email`.


## Description

A RUST service that uses the inbound/outbound capabilities of AWS SES to forwared emails from verfied domains to a verified email address.
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

1. Edit the `terraform/variables.tf` file accordingly to suit your needs.
2. If you're using S3 as your backend, you'll need to make changes to `terraform/main.tf`
3. Test build locally.
```bash
$ cargo build
$ cargo test
```

### Infrastructure Provisioning
1. Verify your domain and email address on SES before running this
2. Create a terraform Token which has admin access to your AWS Account
3. Build and generate your Lambda.zip in the terraform directory
3. Provision infrastructure
```bash
$ cd terraform
$ terraform init
$ terraform validate -json
$ terraform plan
$ terraform apply
```
Alternately, you can run the `release.sh` and it builds your code and provisions your infrastructure.

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
