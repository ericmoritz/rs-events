# rs-events

A web service for events

## Running unit tests

Simply run `make test` to run the unit tests.

## Running BDD Tests

There a suite of Cucumber-js tests in [tests/features](./tests/features).  
These tests use [Docker Compose](https://docs.docker.com/compose/) to run the tests in an isolated environment.

You will need both Docker and Docker Compose in order run the BDD tests.  [Docker Machine](https://docs.docker.com/machine/install-machine/) or [Docker for Mac]()

Simply run `make bdd` to run the tests.
