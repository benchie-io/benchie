#!/bin/sh

set -e

if [ -z "$1" ]; then
    echo "Error: Please add a version in the form of major.minor.patch (e.g. 0.4.0) as argument"
    exit 1
fi

if ! command -v gh >/dev/null; then
	echo "Error: gh (Github CLI) is required to trigger a benchie release." 1>&2
	exit 1
fi

gh workflow run release.yml --ref main -f "version=$1"
