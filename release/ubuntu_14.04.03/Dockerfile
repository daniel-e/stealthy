FROM ubuntu:14.04.3

RUN apt-get update
RUN apt-get -y install wget git build-essential libpcap-dev libssl-dev libncursesw5-dev

WORKDIR /tmp
RUN wget -q https://static.rust-lang.org/dist/rust-1.5.0-x86_64-unknown-linux-gnu.tar.gz
RUN tar xzf rust-1.5.0-x86_64-unknown-linux-gnu.tar.gz
WORKDIR rust-1.5.0-x86_64-unknown-linux-gnu/
RUN ./install.sh

WORKDIR /tmp
RUN echo "#!/bin/bash" > build.sh
RUN echo "git clone https://github.com/daniel-e/stealthy.git" >> build.sh
RUN echo "cd stealthy/" >> build.sh
RUN echo "cargo build" >> build.sh
RUN chmod +x build.sh
