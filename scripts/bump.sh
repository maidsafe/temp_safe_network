#!/bin/bash

set -e -x

# This script needs cargo workspaces installed

for change in $(cargo ws ls) ; do
    echo "CHCH ${change}"
done