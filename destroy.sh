#!/bin/bash
# Stop all containers:
docker kill $(docker ps -q)

# Remove all containers
docker rm $(docker ps -a -q)

# Remove all docker images
docker rmi $(docker images -q)

# Remove all docker volumes:
docker volume ls -qf dangling=true | xargs -r docker volume rm
