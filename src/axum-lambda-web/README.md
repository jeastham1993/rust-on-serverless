# Axum on Lambda

Updated README incoming, for now. You can run locally using Axum & DynamoDB Local by running

```bash
docker-compose up -d
./create-local-table.sh
cargo run
```

When you are ready to deploy to AWS, run

```bash
sam build
sam deploy --guided
```