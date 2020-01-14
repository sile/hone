hone
====

```bash
#!/bin/bash
#
# $ hone init
# $ hone run --study mnist --repeats 10 examples/pytorch-mnist.sh
# $ hone trials mnist | hone best
#
set -eux

SCRIPT_URL=https://raw.githubusercontent.com/pytorch/examples/master/mnist/main.py

LR=$(hone get lr choice 0.001 0.01 0.1 1.0)
GAMMA=$(hone get gamma choice 0.2 0.3 0.4 0.5 0.6 0.7 0.8 0.9)

curl -L $SCRIPT_URL | python -u - --lr=$LR --gamma=$GAMMA --epochs 3 | tee /tmp/mnist.log

grep -oP '(?<=Test set: Average loss: )[0-9.]*' /tmp/mnist.log | tail -1 | xargs hone report
```
