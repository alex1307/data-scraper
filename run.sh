#!/bin/bash

set -e
SOURCE=$1
if [[ -z "$SOURCE" ]]; then
  echo "Usage: $0 <source>"
  exit 1
fi

./crawler.sh --features postgres --source "$SOURCE" --sink-type postgres &