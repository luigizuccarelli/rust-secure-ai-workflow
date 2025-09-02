# Overview

A simple https (secure) command service with http callback 

## Clone and build

```
git clone https://github.com/lmzuccarelli/rust-secure-ai-workflow

cd rust-secure-exec-service

# this assumes you have installed Rust and build essentials

make build
```

## GUI Front End 

This project relies heavily on totaljs to initiate ai-workflows.

Refer to the project [totaljs](https://www.totaljs.com/)

Create your desired flow and point the HTTP Request service/object to this service


## TLS cert creation

Create the private/public key pair and store them in a folder in this directory called ./certs

create a CA authority (self signed)

```
openssl genrsa -out certs/rootCA.key 2048
```

generate root certs

```
openssl req -x509 -new -nodes -key certs/rootCA.key -sha256 -days 1024 -out certs/rootCA.pem -subj "/C=IT/ST=ANCONA/L=ANCONA/O=QUAY/OU=IT Dev/CN=mostro"
```

generate server key

```
openssl genrsa -out certs/ssl.key 2048
```

create a signing request with subject 

```
openssl req -new -key certs/ssl.key -out certs/ssl.csr -subj "/C=IT/ST=ANCONA/L=ANCONA/O=QUAY/OU=IT Dev/CN=mostro"
```

use openssl config to generate ssl cert 

```
openssl x509 -req -in certs/ssl.csr -CA certs/rootCA.pem -CAkey certs/rootCA.key -CAcreateserial -out certs/ssl.cert -days 356 -extensions v3_req -extfile certs/openssl.conf -passin pass:""
```

copy rootCA to system wide trust store

```
sudo cp certs/rootCA.pem /etc/pki/ca-trust/source/anchors/rootCA.pem
```

update trusted store

```
sudo update-ca-trust extract
```
