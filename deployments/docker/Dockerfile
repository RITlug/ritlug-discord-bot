# Dockerfile inspired from: https://hub.docker.com/_/rust/

# How to run:
# With the working directory set to the root of this repository,
# run the following shell command:
# docker build -f deployments/docker/Dockerfile -t ritlug-discord-bot .

# Used for build environments, not for deployment.
# This includes the rust compiler, which probably isn't needed
# in a production environment.
FROM rust:1.64 as builder
WORKDIR /usr/src/ritlug

# Copy only the things needed to compile into
# the build environment, otherwise each time a file changes
# anywhere else in the repo, we'll have to recompile even if no
# source files change.
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo install --path .

# Using Debian's slimmed-down version of buster since we need
# to install additional dependencies, and that's easiest to do in a distro
# with a package manager.
FROM debian:buster-slim

# Need openSSL for the bot to run, so install that.
RUN apt-get update && \
    apt-get install -y openssl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/ritlug /usr/local/bin/ritlug

CMD ["/usr/local/bin/ritlug"]
