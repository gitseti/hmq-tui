#!/bin/bash
curl -X POST http://localhost:8888/api/v1/data-hub/management/start-trial
for i in {1..10}
do
  echo "Creating  'behavior-policy-${i}'"

  JSON=$(jq -n \
  --arg id "behavior-policy-${i}" \
  --arg clientIdRegex "client-${i}" \
  --arg behaviorId "Mqtt.events" \
  '{
    id: $id,
    matching: {
      clientIdRegex: $clientIdRegex
    },
    behavior: {
      id: $behaviorId,
      arguments: {}
    },
  }')

  mqtt hivemq behavior-policy create --definition "${JSON}"
done
