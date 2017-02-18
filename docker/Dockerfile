FROM ubuntu:16.10

RUN apt-get update
RUN apt-get -y install apt-utils
RUN apt-get -y install curl aptitude build-essential
RUN apt-get -y install git

RUN apt-get -y install libpcap-dev libncursesw5-dev libssl-dev ncurses-dev

ADD run.sh /tmp/
CMD ["/bin/bash", "-c", "/tmp/run.sh"]
