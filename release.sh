#!/usr/bin/env bash
# Author: Nyah Check
# Purpose: Build release target for rs boostrap binary
set -eux

echo "Create bootstrap binary"
rustup target add x86_64-unknown-linux-musl
docker pull clux/muslrust
docker run -v $PWD:/volume --rm -t clux/muslrust cargo build --release --target x86_64-unknown-linux-musl
zip -j lambda.zip ./target/x86_64-unknown-linux-musl/release/bootstrap
cp lambda.zip terraform/

echo "Terraform Provisioning"
terraform validate -json terraform
terraplan plan terraform
terraform apply terraform

echo ">>>>>Release complete<<<<"