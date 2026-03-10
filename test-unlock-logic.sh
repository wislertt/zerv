#!/bin/bash

# Test script for get-missing-deploy-labels logic

echo "=========================================="
echo "Test 1: Empty labels (should unlock all)"
echo "=========================================="
LABELS='{}'
LABEL_PREFIX="deploy-"
ENVS='["d", "n", "p"]'
SERVICES='["dummy-service-1", "dummy-service-2"]'

echo "LABELS: $LABELS"
echo "ENVS: $ENVS"
echo "SERVICES: $SERVICES"

COMB_ARRAY=$(echo '{}' | jq -c --argjson labels "$LABELS" \
                            --arg prefix "$LABEL_PREFIX" \
                            --argjson envs "$ENVS" \
                            --argjson services "$SERVICES" \
    '[
      ($envs[] | . as $env | select(($labels | has("\($prefix)\($env)")) | not) |
        $services[] | {"env": $env, "service": .})
    ]')

echo "Result: $COMB_ARRAY"
echo ""

echo "=========================================="
echo "Test 2: With deploy-d label (should NOT unlock d)"
echo "=========================================="
LABELS='{"deploy-d": {}}'
LABEL_PREFIX="deploy-"
ENVS='["d", "n", "p"]'
SERVICES='["dummy-service-1", "dummy-service-2"]'

echo "LABELS: $LABELS"
echo "ENVS: $ENVS"
echo "SERVICES: $SERVICES"

COMB_ARRAY=$(echo '{}' | jq -c --argjson labels "$LABELS" \
                            --arg prefix "$LABEL_PREFIX" \
                            --argjson envs "$ENVS" \
                            --argjson services "$SERVICES" \
    '[
      ($envs[] | . as $env | select(($labels | has("\($prefix)\($env)")) | not) |
        $services[] | {"env": $env, "service": .})
    ]')

echo "Result: $COMB_ARRAY"
echo ""

echo "=========================================="
echo "Test 3: Empty service names (env-only)"
echo "=========================================="
LABELS='{}'
LABEL_PREFIX="deploy-"
ENVS='["d", "n", "p"]'
SERVICES='[""]'

echo "LABELS: $LABELS"
echo "ENVS: $ENVS"
echo "SERVICES: $SERVICES"

COMB_ARRAY=$(echo '{}' | jq -c --argjson labels "$LABELS" \
                            --arg prefix "$LABEL_PREFIX" \
                            --argjson envs "$ENVS" \
                            --argjson services "$SERVICES" \
    '[
      ($envs[] | . as $env | select(($labels | has("\($prefix)\($env)")) | not) |
        $services[] | {"env": $env, "service": .})
    ]')

echo "Result: $COMB_ARRAY"
echo ""
