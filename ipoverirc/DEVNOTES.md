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

# Testing in Docker

sudo docker daemon
sudo docker build --rm -t stealthy/ubuntu16.04 .

Run the image:
sudo docker run --privileged -v /home/dz:/host -t -i stealthy/ubuntu16.04 /bin/bash

git clone https://github.com/daniel-e/stealthy.git
cd stealthy/ipoverirc/
make
> in screen:
./ipoverirc tun0
> in another screen window:
ifconfig tun0 192.168.5.10 netmask 255.255.255.0 up
cd ..
cargo build
./target/debug/stealthy -i tun0 -d 192.168.5.11

Run a second image and do the same but configure the tun network device with another IP:
[...]
ifconfig tun0 192.168.5.11 netmask 255.255.255.0 up
cd ..
cargo build
./target/debug/stealthy -i tun0 -d 192.168.5.10

# Further readings

* https://raw.githubusercontent.com/tkislan/base64/master/base64.h
* https://www.kernel.org/doc/Documentation/networking/tuntap.txt
