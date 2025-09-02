#!/bin/bash

ROOTCA="rootCA"
CERTS_DIR="./certs"
PK="${PK_ID:?PK_ID environment variable must be set}"
USER="${REMOTE_USER:-lzuccarelli}"

AR_IP=('192.168.1.209' '192.168.1.222' '192.168.1.125' '192.168.1.76' '192.168.1.202' '192.168.1.149' '192.168.1.230')
AR_HOST=('vance' 'vinnie' 'grayman' 'bevin' 'lavern' 'lewellyn' 'george')

