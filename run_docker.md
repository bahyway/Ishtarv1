## Run first docker compose to create docker containers:
```
cd RustLAB
docker-compose up -d
```
## To log in the docker container , find first the docker name ( copy the container number):
```
# Find running containers
docker ps
```
Then login:
```
docker exec -it e8fd3537d5f7 bash
```
## To check if rust have been  installed on the rust container , run this command:
```
rustc --version
```
