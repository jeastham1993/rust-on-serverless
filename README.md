# Rust on Serverless

*A short disclaimer, I am learning in public with this repository. The samples may not represent best practices for the Rust language. If you spot anything that could be done better, please reach out and let me know.*

Historically, I've always worked with .NET, an object orientated language. This repo is my central place to challenge this object oriented view on the world and start working with a different language. All this learning is within the context of AWS Serverless technologies.

One of the original use cases for Rust is as an embedded systems programming language. Embedded systems are typically extremely resource contrained. When you consider the pricing and execution model of Lambda, it can also be considered a resource contrained language. Making Rust a perfect fit. That perfect performance fit can be seen in [these benchmarks](https://github.com/aws-samples/serverless-rust-demo).

## Samples

All the samples listed below can be built and deployed in the same way. From the root of the sample directory, run `make build` and then `sam deploy --guided`. For example, if I wanted to deploy the 'HTTP Web Server on Lambda' example I would run:

``` bash
cd src/axum-lambda-web
make build
sam deploy --guided
```

### HTTP Web Server on Lambda

[Link](./src/axum-lambda-web)

An [Axum](https://github.com/tokio-rs/axum) web server running on AWS Lambda.

### Server Side Rendering on Lambda

[Link](./src/axum-lambda-web-server-side-rendering)

An [Axum](https://github.com/tokio-rs/axum) web server running on AWS Lambda. Server side rendering implemented using [Ructe](https://github.com/kaj/ructe).

### Serverless ToDo API

[Link](./src/serverless-todo)

A fulley serverless ToDo API implemented with API Gateway, Lambda & DynamoDB. Demonstrates how to package multiple handlers in the same project.

### Step Functions Lambda

An order validation pipeline implemented using Step Functions & Rust. A look at how functional programming concepts can work with Rust & AWS Serverless technologies.

### SQS -> AWS Lambda

[Link](./src/sqs-sourced-lambda)

A Lambda function sourced by SQS.
