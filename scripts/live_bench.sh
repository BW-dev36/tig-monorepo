#!/bin/bash

# 1. Récupérer l'ID du dernier bloc
block_response=$(curl -s https://mainnet-api.tig.foundation/get-block)

# Extraire l'ID du bloc
block_id=$(echo "$block_response" | jq -r '.block.id')

echo "Block ID: $block_id"

# 2. Récupérer les détails des challenges en utilisant l'ID du bloc
challenges_response=$(curl -s "https://mainnet-api.tig.foundation/get-challenges?block_id=$block_id")

export ALGOS_TO_COMPILE=vector_search_kd_fastdim
export NONCE=0
export WASM=./tig-algorithms/wasm/vector_search/kd_fastdim.wasm
export CHALLENGE=vector_search
export ALGORITHM=kd_fastdim

num_solutions=0
num_invalid=0
num_errors=0
total_ms=0

ON_SETTINGS=$1

echo "Building wasm..."
cargo build -p tig-wasm --target wasm32-wasi --release --features entry-point
wasm-opt target/wasm32-wasi/release/tig_wasm.wasm -o tig-algorithms/wasm/${CHALLENGE}/${ALGORITHM}.wasm -O2 --remove-imports


# 3. Extraire et itérer sur les paires de qualifier_difficulties pour le challenge 'c004'
echo "Processing qualifier difficulties for challenge 'c004'..."
echo "$challenges_response" | jq -c '.challenges[] | select(.id=="c004") | .block_data.qualifier_difficulties[]' | while read -r pair; do
    echo "Processing pair: $pair"
    if [ -n "$ON_SETTINGS" ]; then
        SETTINGS="{\"challenge_id\":\"c004\",\"difficulty\":$ON_SETTINGS,\"algorithm_id\":\"c004\",\"player_id\":\"\",\"block_id\":\"\"}"
    else
        SETTINGS="{\"challenge_id\":\"c004\",\"difficulty\":$pair,\"algorithm_id\":\"c004\",\"player_id\":\"\",\"block_id\":\"\"}"
    fi
    start_time=$(date +%s%3N)
    output=$(./target/release/tig-worker compute_solution --fuel 10000000000 $SETTINGS $NONCE $WASM 2>&1)
    exit_code=$?
    end_time=$(date +%s%3N)
    duration=$((end_time - start_time))
    total_ms=$((total_ms + duration))
    if [ $exit_code -eq 0 ]; then
        num_solutions=$((num_solutions + 1))
    else
      if echo "$output" | grep -q "Invalid solution\|No solution found"; then
          num_invalid=$((num_invalid + 1))
      else
          num_errors=$((num_errors + 1))
      fi
    fi
    if [ $((num_solutions)) -eq 0 ]; then
        avg_ms_per_solution=0
    else
        avg_ms_per_solution=$((total_ms / num_solutions))
    fi

    echo -e "#instances: $((num_solutions + num_invalid + num_errors)), #solutions: $num_solutions, #invalid: $num_invalid, #errors: $num_errors, average ms/solution: $avg_ms_per_solution, $duration duration in ms"

    if [ -n "$ON_SETTINGS" ]; then
        echo -e "$output"
        break
    fi

done