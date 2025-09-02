#!/bin/bash
#

# set -exv 

source ./setup/variables.sh


create_root_ca() {
  rm -rf ${CERTS_DIR}

  openssl genrsa -out ${CERTS_DIR}/${ROOTCA}.key 2048
  # vinnie is the node where nginx is installed
  openssl req -x509 -new -nodes -key ${CERTS_DIR}/${ROOTCA}.key -sha256 -days 1024 -out ${CERTS_DIR}/${ROOTCA}.pem -subj "/C=IT/ST=ANCONA/L=ANCONA/O=WORKFLOWAI/OU=IT Dev/CN=${AR_HOST[1]}"
  openssl genrsa -out ${CERTS_DIR}/ssl.key 2048

  # local
  # sudo cp ${CERTS_DIR}/${ROOTCA}.pem /etc/pki/ca-trust/source/anchors/${ROOTCA}.pem
  # sudo update-ca-trust extract
  
  # remote
  scp -i "${PK}" -r ./certs/root* "${USER}@${AR_IP[1]}:/home/${USER}/certs"
	ssh -i "${PK}" "${USER}@${AR_IP[1]}" -t "sudo cp /home/${USER}/certs/$ROOTCA.pem /etc/pki/ca-trust/source/anchors/ && sudo update-ca-trust extract"

  echo -e "exit => $?"
}

create_certs() {
  # assumes rootCA has been created
	for i in "${!AR_HOST[@]}"; do
	  mkdir -p ${CERTS_DIR}/${AR_HOST[i]}
	  rm -rf ${CERTS_DIR}/${AR_HOST[i]}/*
    tee ${CERTS_DIR}/${AR_HOST[i]}/openssl.conf <<EOF > /dev/null 2>&1
 [req]
req_extensions = v3_req
distinguished_name = req_distinguished_name
[req_distinguished_name]
[ v3_req ]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
subjectAltName = @alt_names
[alt_names]
DNS.1 = ${AR_HOST[i]}
IP.1 = ${AR_IP[i]}
EOF
    openssl req -new -key ${CERTS_DIR}/ssl.key -out ${CERTS_DIR}/${AR_HOST[i]}/ssl.csr -subj "/C=IT/ST=ANCONA/L=ANCONA/O=WORKFLOWAI/OU=IT Dev/CN=${AR_HOST[i]}"
    openssl x509 -req -in ${CERTS_DIR}/${AR_HOST[i]}/ssl.csr -CA ${CERTS_DIR}/${ROOTCA}.pem -CAkey ${CERTS_DIR}/${ROOTCA}.key -CAcreateserial -out ${CERTS_DIR}/${AR_HOST[i]}/ssl.cert -days 356 -extensions v3_req -extfile ${CERTS_DIR}/${AR_HOST[i]}/openssl.conf -passin pass:""
  done

  echo -e "exit => $?"
}

install_certs() {
	for i in "${!AR_HOST[@]}"; do
		ssh -i "${PK}" "${USER}@${AR_IP[i]}" -t "mkdir -p /home/${USER}/certs"
		scp -i "${PK}" -r ./certs/ssl.key "${USER}@${AR_IP[i]}:/home/${USER}/certs"
		scp -i "${PK}" -r ./certs/${AR_HOST[i]}/* "${USER}@${AR_IP[i]}:/home/${USER}/certs"
	done
  echo -e "exit => $?"
}

"$@"
