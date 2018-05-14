** This is a work in progress to explore building services in Rust**

# rs-events

A web service for events

## Running unit tests

Simply run `make test` to run the unit tests.

## Running BDD Tests

There is a suite of Cucumber-js tests in [tests/features](./tests/features).  
These tests use [Docker Compose](https://docs.docker.com/compose/) to run the tests locally in an isolated environment.

You will need both Docker and Docker Compose in order run the BDD tests.  [Docker Machine](https://docs.docker.com/machine/install-machine/).

Simply run `make bdd` to run the tests.
