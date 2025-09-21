#!/bin/bash

USER="${REMOTE_USER:-lzuccarelli}"
PK="${PK_ID:?PK_ID environment variable must be set}"
MS="secure-ai-workflow"
DESCRIPTION="Simple htmx based webconsole written in Rust"
REPO="https://github.com/lmzuccarelli/rust-secure-ai-workflow.git"
REPO_NAME="rust-secure-ai-workflow"
CLEAN=$1

create_configs() {
tee config/${MS}-config.json <<EOF
{
	"name": "rust-secure-ai-workflow",
	"description": "a secure generic script executor for ai workflows",
	"port": "1338",
	"log_level": "debug",
	"certs_dir": "/home/${USER}/certs",
	"cert_mode": "file",
	"db_path" :"/home/${USER}/database",
	"token_url" : "https://vinnie/auth/token",
	"agent_url": "https://vinnie/agent/execute-agent",
	"agents": {
		"simple-ai-agent": "/home/${USER}/services/rust-simple-agent",
		"gemini-ai-agent": "/home/${USER}/services/gemini-ai-agent",
		"llama3b-ai-agent": "/home/${USER}/services/llama3b-ai-agent",
		"rust-coding-agent": "/home/${USER}/services/rust-coding-agent",
		"demo-agent": "/home/lzuccarelli/${USER}/demo-agent"
	}
}
EOF

tee config/${MS}.service <<EOF
[Unit]
Description=${MS}-service

[Service]
ExecStart=/home/${USER}/services/${MS}-service --config /home/${USER}/services/${MS}-config.json
Restart=Always
PIDFile=/tmp/${MS}_service_pid
EOF
}

clone_build_service() {
  HOSTS=("george")
  for host in "${HOSTS[@]}"; do
    ssh -i "${PK}" "${USER}@${host}" -t "rm -rf /home/${USER}/services/${MS}-service"
    eval `ssh-agent`
    ssh-add ~/.ssh/id_ed25519-lz
    if [ "${CLEAN}" == "true" ];
    then
      ssh -i "${PK}" "${USER}@${host}" -tA "rm -rf /home/${USER}/Projects/${REPO_NAME} && cd /home/${USER}/Projects && git clone ${REPO} && cd ${REPO_NAME} && make build"
    else 
      ssh -i "${PK}" "${USER}@${host}" -tA "cd /home/lzuccarelli/Projects/${REPO_NAME} && rm -rf target/release/*secure* && git pull origin main --rebase && make build"
    fi
  done
}

deploy_service() {
  HOSTS=("george")
  for host in "${HOSTS[@]}"; do
    scp -i "${PK}" config/* "${USER}@${host}:/home/${USER}/services"
    ssh -i "${PK}" "${USER}@${host}" -t "cp /home/${USER}/Projects/${REPO_NAME}/target/release/${MS} /home/${USER}/services/${MS}-service"
    ssh -i "${PK}" "${USER}@${host}" -t "sudo cp /home/${USER}/services/${MS}.service /etc/systemd/system/"
  done
}


start_service() {
  ssh -i "${PK}" "${USER}@george" -t "sudo systemctl daemon-reload && sudo systemctl start ${MS}.service"
}

restart_service() {
  ssh -i "${PK}" "${USER}@george" -t "sudo systemctl daemon-reload && sudo systemctl restart ${MS}.service"
}

stop_service() {
  ssh -i "${PK}" "${USER}@george" -t "sudo systemctl daemon-reload && sudo systemctl stop ${MS}.service"
}

"$@"
