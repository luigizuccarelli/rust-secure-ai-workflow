#!/bin/bash

#set -ex 
set -o pipefail

curl -s -X GET "https://mostro:1336/token?user=lmzuccarelli&session-id=123456789" | jq '.access_token' > staging/token

JWTTOKEN=$(cat staging/token)

tee staging/payload.json <<EOF > /dev/null 2>&1
{
  "apiVersion": "api.taskexecute.io/v1",
  "kind": "TaskExecute",
  "spec": {
    "prompt": "Evaluate the following text content 'the code base is in a shambles, refactor it and make sure you use SOLID principles, finally create a PR for the staff engineers to review' \nAs an assistant create an action plan from the text content, use the reference issue LMZ-7239. Be detailed as possible.",
    "file": "action-task-lmz-7239.md",
	  "title" : "Action task overview for LMZ-7239",
    "token": ${JWTTOKEN},
    "nodes": [{
        "name": "localhost",
        "agent": "none",
        "user": "lzuccarelli",
        "consoleLog": true
    }]
  }
}
EOF

echo -e "exit => $?"

