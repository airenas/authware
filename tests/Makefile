-include .env
#####################################################################################
## print usage information
help:
	@echo 'Usage:'
	@cat ${MAKEFILE_LIST} | grep -e "^## " -A 1 | grep -v '\-\-' | sed 's/^##//' | cut -f1 -d":" | \
		awk '{info=$$0; getline; print "  " $$0 ": " info;}' | column -t -s ':' 
.PHONY: help
#####################################################################################
## start all containers
start: 
	docker compose up -d redis
.PHONY: start
## invoke integration tests
test/integration: start 
	docker compose up --build --exit-code-from integration-test authware integration-test
.PHONY: test/integration
## clean everything, stops docker containers and removes them
clean:
	docker compose down
.PHONY: clean	
#####################################################################################
.EXPORT_ALL_VARIABLES:
