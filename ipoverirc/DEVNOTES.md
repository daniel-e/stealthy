# Required IRC commands

NICK <nickname>
USER <user> <mode> <unused> <realname>
JOIN <channel>
PRIVMSG <msgtarget> <text to be sent>

## Examples
 
telnet irc.swiftirc.net 7000
NICK blablabladf
USER adifubasdfiub 0 * :bla sdf
JOIN :#abcabababab
PRIVMSG #abcabababab :msg
QUIT

Receiving a message

:aiusbazuwer!~dz@Swift-CDA1FC85.gw-nat.spb.muc.de.oneandone.net PRIVMSG #abcabababab :x

Sometimes you get a ping from the server
PING :bipartite.ny.us.SwiftIRC.net

Reply with:
PONG :bipartite.ny.us.SwiftIRC.net

# Docker for testing

sudo docker daemon
sudo docker build --rm -t stealthy/ubuntu16.04 .

sudo docker run --privileged -v /home/dz:/host -t -i stealthy/ubuntu16.04 /bin/bash

Guest:
./main tun0
ifconfig tun0 192.168.2.18 netmask 255.255.255.0 up

Host:
./main tun0
ifconfig tun0 192.168.2.17 netmask 255.255.255.0 up


