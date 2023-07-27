#!/usr/bin/env bash

export BRIDGE_URL=$(grep BRIDGE_URL config | cut -f2 -d=)
export CERTBOT_EMAIL=$(grep CERTBOT_EMAIL config | cut -f2 -d=)

sed -e "s#NGINX_IMAGE#${NGINX_IMAGE}#" -e "s#BRIDGE_URL#${BRIDGE_URL}#" \
 -e "s#CERTBOT_EMAIL#${CERTBOT_EMAIL}#" -e "s#REDIS_IMAGE#${REDIS_IMAGE}#" \
 -e "s#BRIDGE_IMAGE#${BRIDGE_IMAGE}#"  ops/docker-compose-template.yml > ops/docker-compose.yml

run="docker-compose -f ops/docker-compose.yml up -d "

printf "\nDeploy command: $run\n\n"
exec $run
