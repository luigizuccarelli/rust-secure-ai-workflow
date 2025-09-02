#!/bin/bash
set -e -E

PROMPT=$1
FILE_NAME=$2
TITLE=$3

cd ~/Projects/rust-ragllm-qdrant-chat/

cp ~/Projects/rust-secure-ai-workflow/results/${FILE_NAME} kb-docs/workflow-ai/

./target/release/rust-ragllm-qdrant-chat --config config.json --override-regex "# ${TITLE}"

# for error handling this format is important
echo -e "exit => $?"
