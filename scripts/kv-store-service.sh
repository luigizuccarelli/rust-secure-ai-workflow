#!/bin/bash
#

set -exv 
KVBIN="/home/lzuccarelli/Projects/rust-kv-store/target/release/rust-kv-store"
CONFIG="/home/lzuccarelli/Projects/rust-kv-store/config/kv.json"

if [ "$1" == "read" ];
then
  $KVBIN --config=${CONFIG} --key $2 read

else
  $KVBIN --config=${CONFIG} --key $2 --value-filepath $3 write
fi
  
# for the error handling this format is important
echo -e "exit => $?"
