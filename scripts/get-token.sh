#!/bin/bash
#
set -exv 

generate_post_data()
{
  cat <<EOF
{
  "apiVersion": "api.taskexecute.io/v1",
  "kind": "TaskExecute",
  "spec": {
    "prompt": "NA",
    "file": "NA",
    "title" : "NA",
    "token": "NA",
    "nodes": [{
        "name": "localhost",
        "agent": "token",
        "user": "lzuccarelli",
        "consoleLog": true
    }]
  }
}
EOF

# curl -s -d "$(generate_post_data)" "https://mostro:1336/token?user=lmzuccarelli&session-id=123456789" | jq '.access_token' > staging/token
curl -s -X GET "https://mostro:1336/token?user=lmzuccarelli&session-id=123456789" | jq '.access_token' > staging/token
  
# for the error handling this format is important
echo -e "exit => $?"
