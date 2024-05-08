#!/bin/bash
docker run --ulimit nofile=500000:500000 -p 8080:8080 -p 8000:8000 -p 1883:1883 --env HIVEMQ_REST_API_ENABLED=true -p 8888:8888 hivemq/hivemq4
