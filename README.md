[![CI/CD](https://github.com/simpleg-eu/cp-microservice/actions/workflows/ci-cd.yml/badge.svg)](https://github.com/simpleg-eu/cp-microservice/actions/workflows/ci-cd.yml)

# Introduction

cp-microservice is meant to be a utility library so you can easily create microservices with Rust. Currently all effort is focused towards AMQP based APIs, although the library can easily be fit to expose REST APIs through HTTP.

## Architecture

The architecture proposed by cp-microservice for Rust microservices is designed around the idea of 3 layers which run in parallel. These layers are the following:

1. API: Here incoming requests are routed and handled accordingly by sending requests to the `Logic` layer.
2. Logic: The business logic resides here. Here the incoming logic requests are handled and whenever there's a need for storage related actions, requests are sent to the storage layer.
3. Storage: Here storage requests are handled by doing direct calls to the database or whatever storage system is being used.
