To run:

First time:
```bash
docker compose up
# Setup the development database
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate up
```
When running again you can just do
```bash
docker container kill --signal USR1 yet-to-be-named-golfing-site-yq-runner-1
```
