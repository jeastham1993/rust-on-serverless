# Learning Rust

Historically, I've always worked with .NET. An object orientated language. This repo is my central place to challenge this OOP view on the world and start working with a different language.

As Rust is intended as a C/C++ like language, isn't particularly object orientated and generally couldn't possibly be further from .NET it seemed a logical choice. Throw in the performance, especially in resource constrained environments, and my serverless brain is intrigued.

## Samples

### HTTP Web Server

[Link](./src/actix-gcd)

A simple HTTP web server implemented in Rust.

### API Gateway -> AWS Lambda

[Link](./src/http-sourced-lambda)

A Lambda function implemented in Rust sourced by a API Gateway HTTP API.

### SQS -> AWS Lambda

[Link](./src/sqs-sourced-lambda)

A Lambda function sourced by SQS.
