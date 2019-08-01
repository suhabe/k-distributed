#!/bin/bash

set -e

cd ~/k-distributed/kworker && cargo build
cd ~/k-distributed/kdocker
cp ../kworker/target/debug/worker kworker_target
docker build -t kworker -f Dockerfile-bionic.src .
docker tag kworker:latest 626351541105.dkr.ecr.us-east-2.amazonaws.com/kworker:latest
`aws ecr get-login --no-include-email`
docker push 626351541105.dkr.ecr.us-east-2.amazonaws.com/kworker:latest