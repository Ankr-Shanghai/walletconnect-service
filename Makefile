### Deploy configs
BRANCH=$(shell git rev-parse --abbrev-ref HEAD)
REMOTE="https://github.com/Ankr-Shanghai/walletconnect-service"
REMOTE_HASH=$(shell git ls-remote $(REMOTE) $(BRANCH) | head -n1 | cut -f1)
project=walletconnect-service
redisImage='docker.dragonflydb.io/dragonflydb/dragonfly'
nginxImage='$(project)/nginx:$(BRANCH)'
walletConnectImage='$(project)/bridge:$(BRANCH)'

### Makefile internal coordination
flags=.makeFlags
VPATH=$(flags)

$(shell mkdir -p $(flags))

.PHONY: all clean default
define DEFAULT_TEXT
Available make rules:

pull:\tdownloads docker images

setup:\tconfigures domain an certbot email

build:\tbuilds docker images

deploy:\tdeploys to production

stop:\tstops all walletconnect docker stacks

upgrade:
\tpulls from remote git. Builds the containers and updates each individual
\tcontainer currently running with the new version that was just built.

clean:\tcleans current docker build

reset:\treset local config
endef

### Rules
export DEFAULT_TEXT
default:
	@echo -e "$$DEFAULT_TEXT"

pull:
	docker pull $(redisImage)
	@touch $(flags)/$@
	@echo "MAKE: Done with $@"
	@echo

setup:
	@read -p 'Bridge URL domain: ' bridge; \
	echo "BRIDGE_URL="$$bridge > config
	@read -p 'Email for SSL certificate (default noreply@gmail.com): ' email; \
	echo "CERTBOT_EMAIL="$$email >> config
	@touch $(flags)/$@
	@echo "MAKE: Done with $@"
	@echo

build-node: pull
	docker build \
		-t $(walletConnectImage) \
		--build-arg BRANCH=$(BRANCH) \
		--build-arg REMOTE_HASH=$(REMOTE_HASH) \
		-f ops/node.Dockerfile .
	@touch $(flags)/$@
	@echo "MAKE: Done with $@"
	@echo

build-nginx: pull
	docker build \
		-t $(nginxImage) \
		--build-arg BRANCH=$(BRANCH) \
		--build-arg REMOTE_HASH=$(REMOTE_HASH) \
		-f ops/nginx/nginx.Dockerfile ./ops/nginx
	@touch $(flags)/$@
	@echo  "MAKE: Done with $@"
	@echo

build: pull build-node build-nginx
	@touch $(flags)/$@
	@echo  "MAKE: Done with $@"
	@echo
redeploy: 
	$(MAKE) clean
	$(MAKE) down

deploy: setup build 
	BRIDGE_IMAGE=$(walletConnectImage) \
	NGINX_IMAGE=$(nginxImage) \
	PROJECT=$(project) \
        REDIS_IMAGE=$(redisImage) \
	bash ops/deploy.sh
	@echo  "MAKE: Done with $@"
	@echo

down: stop

stop: 
	docker-compose -f ops/docker-compose.yml down
	@echo  "MAKE: Done with $@"
	@echo

reset:
	$(MAKE) clean-all
	rm -f config
	rm -f ops/docker-compose.yml
	@echo  "MAKE: Done with $@"
	@echo

clean:
	rm -rf .makeFlags/build*
	@echo  "MAKE: Done with $@"
	@echo

clean-all:
	rm -rf .makeFlags
	@echo  "MAKE: Done with $@"
	@echo
