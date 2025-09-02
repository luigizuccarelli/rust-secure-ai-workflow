#!/bin/bash

source ./setup/variables.sh

REPO="git@github.com:luigizuccarelli/rust-auth-microservice.git"
REPO_NAME="rust-auth-microservice"
MS="auth"
DESCRIPTION="A simple jwt microservice for auth"
PORT="1336"
BASE_DIR="./setup/microservices/${MS}"
CREATE="false"

# setup sepcific config for auth
create_configs() {
	mkdir -p "${BASE_DIR}/config"

tee "${BASE_DIR}/config/${MS}-config.json" <<EOF > /dev/null 2>&1
{
	"name": "${MS}-microservice",
	"description": "${DESCRIPTION}",
	"port": "${PORT}",
	"certs_dir": "/home/${USER}/certs",
	"cert_mode": "file",
	"jwt_duration": "3600",
	"issuer": "https://samcopai.com",
	"audience": "samcopai",
	"subject": "system auth claim",
	"log_level": "debug",
	"user_api_url": "https://sessions.srv.quicktable.co/"
}
EOF
}

# this section will be common t oall microservices
create_service() {
tee "${BASE_DIR}/config/${MS}.service" <<EOF > /dev/null 2>&1
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
		ssh -i "${PK}" "${USER}@${host}" -t "mkdir -p /home/${USER}/certs && mkdir -p /home/${USER}/microservices && rm -rf /home/${USER}/microservices/*"
		# compile on remote machine
		if [ "${CREATE}" = "true" ];
		then
			ssh -i "${PK}" "${USER}@${host}" -tA "ssh-add /home/${USER}/.ssh/id_ed25519-lz && cd /home/${USER}/Projects && rm -rf ${REPO_NAME} && git clone ${REPO}  && cd ${REPO_NAME} && make build"
    else			
			ssh -i "${PK}" "${USER}@${host}" -t "cd /home/${USER}/Projects/${REPO_NAME} && make build"
		fi 
		# using same machine to execute ms
		ssh -i "${PK}" "${USER}@${host}" -t "cp ./target/release/${MS}-microservice /home/${USER}/microservices"
		scp -i "${PK}" -r ${BASE_DIR}/config/* "${USER}@${host}:/home/${USER}/microservices"
		ssh -i "${PK}" "${USER}@${host}" -t "sudo cp /home/${USER}/microservices/${MS}.service /etc/systemd/system/"
	done
  echo -e "exit => $?"
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
