#!/bin/bash

source ./setup/variables.sh 

USER="lzuccarelli"
PK="${PK_ID:?PK_ID environment variable must be set}"


install_proxy() {
	ssh -i "${PK}" "${USER}@${AR_IP[1]}" -t "mkdir -p /home/${USER}/config && sudo apt-get install nginx -y && sudo systemctl enable nginx"
}

copy_configs() {
	rm -rf ./setup/nginx/nginx-hosts
	echo "127.0.0.1	localhost" > ./setup/nginx/nginx-hosts
	echo "::1		localhost ip6-localhost ip6-loopback" >> ./setup/nginx/nginx-hosts
	echo "ff02::1		ip6-allnodes" >> ./setup/nginx/nginx-hosts
	echo "ff02::2		ip6-allrouters" >> ./setup/nginx/nginx-hosts
	for i in "${!AR_HOST[@]}"; do
		echo -e "${AR_IP[i]} ${AR_HOST[i]}" >> ./setup/nginx/nginx-hosts
	done 
	scp -i "${PK}" -r ./setup/nginx/nginx* "${USER}@${AR_IP[1]}:/home/${USER}/config"
	ssh -i "${PK}" "${USER}@${AR_IP[1]}" -t "sudo cp /home/${USER}/config/nginx.conf /etc/nginx/"
	ssh -i "${PK}" "${USER}@${AR_IP[1]}" -t "cat /home/${USER}/config/nginx-hosts | sudo tee /etc/hosts"
}


start_proxy() {
	ssh -i "${PK}" "${USER}@${AR_IP[1]}" -t "sudo systemctl start nginx"
}

restart_proxy() {
	ssh -i "${PK}" "${USER}@${AR_IP[1]}" -t "sudo systemctl restart nginx"
}

stop_proxy() {
	ssh -i "${PK}" "${USER}@${AR_IP[1]}" -t "sudo systemctl stop nginx"
}

"$@"


