#!/bin/bash
#
# $ hone run --repeat 10 examples/pytorch-mnist.sh
#
set -eux

SCRIPT_URL=https://raw.githubusercontent.com/pytorch/examples/master/mnist/main.py

LR=$(hone hp lr choice 0.001 0.01 0.1 1.0)
GAMMA=$(hone hp gamma choice 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9)

curl -L $SCRIPT_URL | python3 -u - --lr=$LR --gamma=$GAMMA --epochs 3 | tee /tmp/mnist.log

grep -oP '(?<=Test set: Average loss: )[0-9.]*' /tmp/mnist.log | tee /dev/stderr | tail -1 | xargs hone tell
