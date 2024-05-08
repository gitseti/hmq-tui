#!/bin/bash
FILE="$(dirname -- "$0")/generic_json_schema.json"
for i in {1..10}
do
  echo "Creating schema 'json-${i}'"
  mqtt hivemq schema create --id json-"${i}" --type json --file "${FILE}"
done
