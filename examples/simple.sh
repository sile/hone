#!/bin/bash
set -eux

echo "PID: $$"

X=$(hone ask x range 1 100 --step 1)
Y=$(hone ask y choice 2 3 4 5)

hone tell minimize $(($X * $Y))
