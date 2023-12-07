# Axum on Lambda

This application demonstrates how to run an Axum web API on AWS Lambda using the [Lambda Web Adapter](https://github.com/awslabs/aws-lambda-web-adapter) project.

It also demonstrates how the principles of clean architecture and domain driven design can be applied to a Rust application. The application is split into 2 parts:

1. main.rs: The main application entrypoint, containing all the logic to configure API specific routes and configuration. This layer also initializes SDK's and the various implementation details like message buses and databases
2. application: contains all of the domain logic. A combination of private and public functionsm, structs and traits that only expose the neccessary functionality to the API layer. For example, the `ToDo` struct is never exposed outside of this crate, instead a DTO named `ToDoItem` is the interface

This example is work in progress, and may not actually be something idiomatic in the Rust ecosystem. However, the combination of traits, structs, enums and pattern matching make this a powerful way to define business logic and build applications that incorporate domain logic.

## Run Locally

This application supports running locally, with DynamoDB local for persistence and an `InMemoryMessagePublisher` to simulate events being sent to EventBridge.

```bash
docker-compose up -d
./create-local-table.sh
export USE_LOCAL=Y
export TABLE_NAME=TODO
cargo run
```

## Test

The application contains a suite of tests, at all layers of the stack. The tests defined in [main.rs](./src/main.rs) create the actual request router used by Axum and use that to send requests directly into Axum.

All tests defined in the [application](./src/application/) layer use mock or in-memory implementations to substitute database and message publishing functionality.

To run tests:

```bash
cargo test
```

## Deploy

AWS SAM is used to deploy this application to AWS.

```bash
sam build
sam deploy --guided
```

[Blog Post discussing the ideas](https://jameseastham.co.uk/post/software-development/hexagaonal-architecture-rust/)