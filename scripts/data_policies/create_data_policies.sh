#!/bin/bash
curl -X POST http://localhost:8888/api/v1/data-hub/management/start-trial
for i in {1..10}
do
  echo "Creating  'data-policy-${i}'"

  JSON=$(jq -n \
  --arg id "data-policy-${i}" \
  --arg topicFilter "topic-${i}" \
  '{
    id: $id,
    matching: {
      topicFilter: $topicFilter
    },
  }')

  mqtt hivemq data-policy create --definition "${JSON}"
done
