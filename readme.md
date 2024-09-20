To run:

First time:
```bash
cargo build
docker build -t yg
docker run -v `realpath target/debug`:/debug -it --name yg --security-opt seccomp=unconfined yg
```
When running again you can just do
```bash
docker container start -a yg
```
