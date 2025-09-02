# Overview

## pre-requistes

### selinux

ensure mount points are setup correctly

```
# as an example
sudo semanage fcontext -a -t container_file_t '/home/student/local/mysql(/.*)?'
sudo restorecon -R /home/student/local/mysql
ls -Z /home/student/local/mysql

# output should like something like this 
system_u:object_r:container_file_t:s0

```

### execute

```
sudo podman run --rm --name nginx -v ./setup/nginx.conf:/etc/nginx/nginx.conf -v ./certs:/etc/nginx/certs -p 443:443 -d  nginx

```
