# IP over IRC

This directory contains a tool which can be used to send IP packets over IRC. The tool registers a virtual network device over which IP packets can be routed. The IP packets that are routed over this device are encoded as base64 and are sent to the channel `#ipoverirc` on the SwiftIRC network (irc.swiftirc.net:7000).

Using the tool is quite easy.

## Checkout the sources and compile them

```bash
git clone https://github.com/daniel-e/stealthy.git
cd stealthy/ipoverirc/
make
```

## Run and setup

Start the binary ipoverirc.

```bash
sudo ./ipoverirc tun0
```

This will create the network device tun0. To use this device you have to configure it with standard Linux tools. For example, to assign this device the IP address 192.168.2.10 execute the following command.


```bash
sudo ifconfig tun0 192.168.2.10 netmask 255.255.255.0 up
```

*That's it!*

## Testing

Now, you could do the same steps on a different computer and assign this computer the IP address 192.168.2.11. You could test the connection with ping:


```bash
xx@T:~$ ping -c3 192.168.2.11
PING 192.168.2.11 (192.168.2.11) 56(84) bytes of data.
64 bytes from 192.168.2.11: icmp_seq=1 ttl=64 time=1265 ms
64 bytes from 192.168.2.11: icmp_seq=2 ttl=64 time=1182 ms
64 bytes from 192.168.2.11: icmp_seq=3 ttl=64 time=1155 ms

--- 192.168.2.11 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss, time 2008ms
rtt min/avg/max/mdev = 1155.945/1201.323/1265.041/46.404 ms, pipe 2
```

In channel `#ipoverirc` on irc.swiftirc.net:7000 you will see the following messages:
```
<STEALTHYhiiyurdgudlvidsqzxxpnz> RQAAVEdDQABAAW4AwKgCCsCoAgsIAHg9XtIAAQzOrVcAAAAAnfYKAAAAAAAQERITFBUWFxgZGhscHR4fICEiIyQlJicoKSorLC0uLzAxMjM0NTY3
<STEALTHYdgrdmdjlazlfmrxgkgbemy> RQAAVCkxAABAAcwSwKgCC8CoAgoAAIA9XtIAAQzOrVcAAAAAnfYKAAAAAAAQERITFBUWFxgZGhscHR4fICEiIyQlJicoKSorLC0uLzAxMjM0NTY3
<STEALTHYhiiyurdgudlvidsqzxxpnz> RQAAVEd0QABAAW3PwKgCCsCoAgsIAA8hXtIAAg3OrVcAAAAABRILAAAAAAAQERITFBUWFxgZGhscHR4fICEiIyQlJicoKSorLC0uLzAxMjM0NTY3
<STEALTHYdgrdmdjlazlfmrxgkgbemy> RQAAVCmDAABAAcvAwKgCC8CoAgoAABchXtIAAg3OrVcAAAAABRILAAAAAAAQERITFBUWFxgZGhscHR4fICEiIyQlJicoKSorLC0uLzAxMjM0NTY3
<STEALTHYhiiyurdgudlvidsqzxxpnz> RQAAVEgWQABAAW0twKgCCsCoAgsIAOYbXtIAAw7OrVcAAAAALRYLAAAAAAAQERITFBUWFxgZGhscHR4fICEiIyQlJicoKSorLC0uLzAxMjM0NTY3
<STEALTHYdgrdmdjlazlfmrxgkgbemy> RQAAVCmTAABAAcuwwKgCC8CoAgoAAO4bXtIAAw7OrVcAAAAALRYLAAAAAAAQERITFBUWFxgZGhscHR4fICEiIyQlJicoKSorLC0uLzAxMjM0NTY3
```

Here, `STEALTHYhiiyurdgudlvidsqzxxpnz` sends the ICMP echo requests and `STEALTHYdgrdmdjlazlfmrxgkgbemy` answers with the ICMP echo reply.
