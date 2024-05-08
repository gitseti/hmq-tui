#!/bin/bash
FILE="$(dirname -- "$0")/script.js"
for i in {1..10}
do
  echo "Creating script 'script-${i}'"
  mqtt hivemq script create --id script-"${i}" --file "$FILE" --type transformation
done
