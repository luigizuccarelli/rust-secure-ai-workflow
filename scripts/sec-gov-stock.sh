#!/bin/bash

#set -ex 
set -o pipefail

STOCKSTOKEN=$(cat ~/.config/sec-api/key)
URL="https://api.sec-api.io?token="
DATE=$(date --date "-1 days" +'%Y-%m-%d')

rm -rf staging/*

#if [ -f staging/results.json ]; 
#then
#    rm -rf staging/stocks-result.json
#fi

#if [ -f staging/stocks-payload.json ]; 
#then
#    rm -rf staging/stocks-payload.json
#fi

#if [ -f staging/payload.json ]; 
#then
#    rm -rf staging/payload.json
#fi

#if [ -f staging/payload.json ]; 
#then
#    rm -rf staging/payload.json
#fi

tee staging/stocks-payload.json <<EOF > /dev/null 2>&1
{

  "query": "\"formType\":\"4\" AND \"formType\":(NOT \"N-4\") AND \"formType\":(NOT \"4/A\") AND \"filedAt\":[ ${DATE} ]",
  "from": "0",
  "size": "20",
  "sort": [{ "filedAt": { "order": "desc" } }]
}
EOF


curl -s -d'@staging/stocks-payload.json'  "${URL}${STOCKSTOKEN}" | jq > staging/stocks-result.json
# only consider the values greater that 4
jq '.filings | .[] | select(.formType == "4") | select(.ticker != "") | .ticker' staging/stocks-result.json | sort | uniq -c | sort -k1 -n -r | awk '{ if ($1>4) print $1":"$2 }' > staging/stocks.txt

if [ ! -s staging/stocks.txt ]; 
then
  rm -rf staging/*
  echo -e "no stocks to analyze"
  echo -e "exit => 1"
  exit 1
fi

curl -s -X GET "https://mostro:1336/token?user=lmzuccarelli&session-id=123456789" | jq '.access_token' > staging/token

JWTTOKEN=$(cat staging/token)
# echo -e "token : ${JWTTOKEN}"

if [ "${JWTTOKEN}" == "null" ] || [ -z "${JWTTOKEN}" ];
then
  rm -rf staging/*
  echo -e "could not get token"
  echo -e "exit => 2"
  exit 2
fi

  
STOCKS=$(cat staging/stocks.txt | cut -d: -f2 | tr '\n' ',' | sed 's/"//g')

tee staging/payload.json <<EOF > /dev/null 2>&1
{
  "apiVersion": "api.taskexecute.io/v1",
  "kind": "TaskExecute",
  "spec": {
    "prompt": "Evaluate the following stocks ${STOCKS} Give an overview of the type of business sector they are in and also if these stocks would a good buy",
    "file": "stock-alert-${DATE}.md",
    "title" : "Stock Alert for ${DATE}",
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

