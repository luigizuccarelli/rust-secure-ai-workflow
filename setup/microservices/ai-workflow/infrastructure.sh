#!/bin/bash

source ./setup/variables.sh

USER="${REMOTE_USER:-lzuccarelli}"
PK="${PK_ID:?PK_ID environment variable must be set}"
REPO="git@github.com:luigizuccarelli/rust-secure-ai-workflow.git"
REPO_NAME="rust-secure-ai-workflow"
MS="ai-workflow"
DESCRIPTION="Used to ensure secure launch of ai workloads and other services"
PORT="1336"
BASE_DIR="./setup/microservices/${MS}"
CREATE="true"

create_configs() {
	mkdir -p "${BASE_DIR}/config"
tee "${BASE_DIR}/config/${MS}-config.json" <<EOF
{
	"name": "${MS}-microservice",
	"description": "${DESCRIPTION}",
	"port": "${PORT}",
	"certs_dir": "/home/${USER}/certs",
	"cert_mode": "file",
	"log_level": "debug",
}
EOF

tee "${BASE_DIR}/config/${MS}.service" <<EOF
[Unit]
Description=${MS}-microservice

[Service]
ExecStart=/home/${USER}/microservices/${MS}-microservice --config /home/${USER}/microservices/${MS}-config.json
Restart=Always
PIDFile=/tmp/${MS}_pid
EOF
}

deploy() {
	# using george
	HOSTS=("${AR_HOST[6]}")
	for host in "${HOSTS[@]}"; do
		ssh -i "${PK}" "${USER}@${host}" -t "mkdir -p /home/${USER}/certs && mkdir -p /home/${USER}/microservices && rm -rf /home/${USER}/microservices/* && mkdir -p /home/${USER}/Projects"
		# compile on remote machine
		if [ "${CREATE}" = "true" ];
		then
			ssh -i "${PK}" "${USER}@${host}" -tA "ssh-add /home/${USER}/.ssh/id_ed25519-lz && cd /home/${USER}/Projects && rm -rf ${REPO_NAME} && git clone ${REPO}  && cd ${REPO_NAME} && make build"
    else			
			ssh -i "${PK}" "${USER}@${host}" -t "cd /home/${USER}/Projects/${REPO_NAME} && make build"
		fi 
		# using same machine to execute ms
		ssh -i "${PK}" "cp ./target/release/${MS}-microservice" "${USER}@${host}:/home/${USER}/microservices"
		scp -i "${PK}" -r ${BASE_DIR}/config/* "${USER}@${host}:/home/${USER}/microservices"
		ssh -i "${PK}" "${USER}@${host}" -t "sudo cp /home/${USER}/microservices/${MS}.service /etc/systemd/system/"
	done
}

start_service() {
	ssh -i "${PK}" "${USER}@${AR_HOST[6]}" -t "sudo systemctl daemon-reload && sudo systemctl start ${MS}.service"
}

restart_service() {
	ssh -i "${PK}" "${USER}@${AR_HOST[6]}" -t "sudo systemctl restart ${MS}.service"
}

stop_service() {
	ssh -i "${PK}" "${USER}@${AR_HOST[6]}" -t "sudo systemctl stop ${MS}.service"
}

"$@"
