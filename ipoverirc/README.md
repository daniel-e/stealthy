# IP over IRC

This directory contains a tool which can be used to send IP packets over IRC. The tool registers a virtual network device over which IP packets can be routed. The IP packets that are routed over this device are encoded as base64 and are sent to the channel #ipoverirc on the SwiftIRC network (irc.swiftirc.net:7000).

Using the tool is quite easy.

## Checkout the sources and compile them

```bash
git clone https://github.com/daniel-e/stealthy.git
cd stealthy/ipoverirc/
make```

## Run and setup

Start the binary ipoverirc.

```bash
sudo ./ipoverirc tun0
```

This will create the network device tun0. To use this device you have to configure it with standard Linux tools. For example, to assign this device the IP address 192.168.2.10 execute the following command.


```bash
sudo ifconfig tun0 192.168.2.10 netmask 255.255.255 up```

