FROM ubuntu:16.04

RUN apt-get update
RUN apt-get -y install wget git build-essential libpcap-dev libssl-dev libncursesw5-dev libboost-all-dev aptitude net-tools screen iputils-ping curl vim sudo telnet netcat

# Install Rust
RUN curl -sSf https://static.rust-lang.org/rustup.sh | sh

