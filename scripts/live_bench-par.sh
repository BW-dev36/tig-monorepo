#!/bin/bash

# 1. Récupérer l'ID du dernier bloc
block_response=$(curl -s https://mainnet-api.tig.foundation/get-block)

# Extraire l'ID du bloc
block_id=$(echo "$block_response" | jq -r '.block.id')

echo "Block ID: $block_id"

# 2. Récupérer les détails des challenges en utilisant l'ID du bloc
challenges_response=$(curl -s "https://mainnet-api.tig.foundation/get-challenges?block_id=$block_id")
export ALGORITHM_LOCAL=${ALGORITHM:-optimax_gpu}
export CHALLENGE_LOCAL=${CHALLENGE:-vector_search}

export ALGOS_TO_COMPILE=${CHALLENGE_LOCAL}_${ALGORITHM_LOCAL}
export WASM=./tig-algorithms/wasm/${CHALLENGE_LOCAL}/${ALGORITHM_LOCAL}.wasm

#For wasm building
export ALGORITHM=${ALGORITHM_LOCAL}
export CHALLENGE=${CHALLENGE_LOCAL}
echo "Challenge ${CHALLENGE} Algorithm = ${ALGORITHM}"
num_solutions=0
num_invalid=0
num_errors=0
total_ms=0

export ON_SETTINGS=$1

echo "Building wasm..."
cargo build -p tig-wasm --target wasm32-wasi --release --features entry-point
wasm-opt target/wasm32-wasi/release/tig_wasm.wasm -O3 -o tig-algorithms/wasm/${CHALLENGE_LOCAL}/${ALGORITHM_LOCAL}.wasm  --remove-imports

# 3. Extraire et itérer sur les paires de qualifier_difficulties pour le challenge 'c004'
echo "Processing qualifier difficulties for challenge 'c004'..."

# Extraire les difficultés pour le challenge 'c004'
difficulties=$(echo "$challenges_response" | jq -c '.challenges[] | select(.id=="c004") | .block_data.qualifier_difficulties[]')

# Fichiers temporaires pour stocker les résultats
export success_file=$(mktemp)
export invalid_file=$(mktemp)
export error_file=$(mktemp)

# Fonction pour exécuter la tâche avec un nonce donné
run_task() {
    #set -x
    pair=$1
    NONCE=$(shuf -i 0-100000000 -n 1)  # Générer un nonce aléatoire
    ALGORITHM_LOCAL="optimax_gpu"
    CHALLENGE_LOCAL="vector_search"

    ALGOS_TO_COMPILE=${CHALLENGE_LOCAL}_${ALGORITHM_LOCAL}
    WASM=./tig-algorithms/wasm/${CHALLENGE_LOCAL}/${ALGORITHM_LOCAL}.wasm

    #For wasm building
    ALGORITHM=$ALGORITHM_LOCAL
    CHALLENGE=$CHALLENGE_LOCAL
    if [ -n "$ON_SETTINGS" ]; then
        echo "Processing pair: $ON_SETTINGS"
        SETTINGS="{\"challenge_id\":\"c004\",\"difficulty\":$ON_SETTINGS,\"algorithm_id\":\"c004\",\"player_id\":\"\",\"block_id\":\"\"}"
    else
        echo "Processing pair: $pair"
        SETTINGS="{\"challenge_id\":\"c004\",\"difficulty\":$pair,\"algorithm_id\":\"c004\",\"player_id\":\"\",\"block_id\":\"\"}"
    fi

    start_time=$(date +%s%3N)
    output=$(./target/release/tig-worker compute_solution --fuel 2000000000 $SETTINGS $NONCE $WASM 2>&1)
    exit_code=$?
    end_time=$(date +%s%3N)
    duration=$((end_time - start_time))

    if [ $exit_code -eq 0 ]; then
        echo -e "$output"
        echo "SUCCESS: Solution found with Nonce: $NONCE"
        echo "$pair,$NONCE,SUCCESS,$duration" >> "$success_file"  # Stocker succès en CSV
    else
        if echo "$output" | grep -q "Invalid solution\|No solution found"; then
            echo "INVALID: No valid solution found for Nonce: $NONCE"
            echo "$pair,$NONCE,INVALID,$duration" >> "$invalid_file"  # Stocker invalides en CSV
        else
            echo -e "ERROR: $output"
            echo "$pair,$NONCE,ERROR,$duration" >> "$error_file"  # Stocker erreurs en CSV
        fi
    fi

    echo -e "Pair: $pair, Nonce: $NONCE, Duration: $duration ms"
    if [ -n "$ON_SETTINGS" ]; then
        return
    fi
}

export -f run_task  # Export the function for parallel

# Utiliser parallel pour exécuter la fonction en parallèle avec 10 essais par difficulté
echo "$difficulties" | parallel --no-notice --will-cite -j 1 '
    seq 50 | parallel -j 31 run_task {}
' ::: "$difficulties"

# Récapitulatif final
num_success=$(wc -l < "$success_file")
num_invalid=$(wc -l < "$invalid_file")
num_errors=$(wc -l < "$error_file")
total=$((num_success + num_invalid + num_errors))

# Affichage des statistiques
echo -e "\n# Final Statistics"
echo -e "Total instances: $total"
echo -e "Success: $num_success"
echo -e "Invalid: $num_invalid"
echo -e "Error: $num_errors"

# Générer un fichier CSV final avec toutes les entrées
final_csv="results.csv"
{
    echo "difficulty,nonce,status,duration_ms"
    cat "$success_file"
    cat "$invalid_file"
    cat "$error_file"
} > "$final_csv"

# Afficher un message indiquant où est le fichier CSV
echo -e "\nResults saved in CSV format to: $final_csv"

# Nettoyer les fichiers temporaires
rm "$success_file" "$invalid_file" "$error_file"
