FROM rust:1.23.0

RUN apt-get update
RUN apt-get -y install libssl-dev libpq-dev default-libmysqlclient-dev libsqlite3-0 libsqlite3-dev


# Create a dummy project that has the same deps to cache them
WORKDIR /usr/src/
RUN USER=root cargo new --bin rs-events
WORKDIR /usr/src/rs-events

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN cargo install diesel_cli
RUN rm -rf ./src

# Copy the source and do the build
COPY src/ src/
COPY ./migrations ./migrations
COPY ./tests/wait-for-it.sh ./wait-for-it.sh
RUN cargo build --release
