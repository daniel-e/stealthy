Stealthy enhancement proposal 

SEP    : 0
Title  : Alternative communication infrastructure
Status : Draft
Type   : Standard
Created: 2016/07/22
Updated: 2016/07/25

INTRODUCTION
===============================================================================

Currently stealthy relies on existing infrastructure (e.g. LAN or WLAN) to send
messages to other chat partners. If this infrastructure does not exist (e.g.
due to ip filters or network failures) stealthy cannot be used. Especially in
those situations it is absolutely necessary that communication is possible. For
example, if infrastructure is broken due to disasters communication is an
important requirement to coordinate rescue. If someone blocks communication
(e.g. by totalitarian regimes) communication could help to escape political
persecution.

Hence, stealthy should not only rely on existing infrastructure. Rather,
stealthy needs an infrastructure that cannot be under the control of some third
party (to mitigate the risk that an administrator can block the traffic) and
that cannot be damaged by a disaster.

This infrastructure should also be cheap and should not rely on any special
hardware as this has the advantage to be available in countries with great
poverty as well.

The solution described in this proposal uses the sound card of a computer and
a simple transmitter and receiver to send and receive data. The solution
has the advantages that first, a sound card is integrated in almost all
computers nowadays. It is quite robust and can be controlled very easily.
Second, the transmitter and receiver are made up of standard components that
are very cheap. Usually the components required for this solution can also be
found in radios or other electronic devices.

The sender transmits the data using a carrier frequency of 1MHz. This has the
advantage that the data can also be received by a standard medium wave radio.
In this case only the transmitter has to be build. If the radio has a line-out
output it could be connected with the line-in input of the sound card. If
a line-out output does not exist the data could be received from the radio's
speaker via a microphone connected to the sound card. 

    Option 1: Connect radio's line-out with sound card's line-in
   
    +----------------+                        +----------------+
    | radio          |                        |     sound card |
    |                |                        |                |
    |       line out o------------------------o line in        |
    +----------------+                        +----------------+
   
    Option 2: Using a microphone
   
    +----------------+                        +----------------+
    | radio          |                        |     sound card |
    |           #### |  microphone            |                |
    | speaker > #### |  O====.----------------o microphone in  |
    +----------------+                        +----------------+
   
    Fig. 1: How to connect a radio with a sound card

The drawback of the pretty low carrier frequency is the wavelength lambda which
becomes quite large. For 1MHz the length lambda is about 300m. A 1/4 lambda
antenna would have a length of 75m which is not practical for a small
transmitter. The following table gives an overview of different possible antenna
lenghts.

    +---------+--------------------+
    | Lambda  |  Length of antenna |
    +---------+--------------------+
    | 1/32    |  9.38m             |
    | 1/64    |  4.68m             |
    | 1/128   |  2.34m             |
    | 1/256   |  1.17m             |
    +---------+--------------------+

    Table 1: Different antenna lenghts



STEALTHY PACKET TRANSMISSION OVER AIR
===============================================================================

* carrier frequency = 1MHz
* for each data byte: start bit + 8 data bits + 2 zero bits (11 bit)
* start bit is used to signal the beginning of the transmission of 8 data bits and to
  reduce effects caused by clock drifts
* 11 bit in 11ms -> 90 byte per second
 
     S   0   1   0   1   1   0   0   1        
   +---+   +---+   +-------+       +---+
   |   |   |   |   |       |       |   |
   |   |   |   |   |       |       |   |
 --+   +---+   +---+       +-------+   +-------- ...
   |   |   |   |   |   |   |   |   |   |   |   |
    1ms

 TODO use some error correcting code?

STEALTHY PACKETS
===============================================================================

 network byte order = big endian (most significant byte first)
 
 Data packet

 +--------+--------+--------+--------+
 |   0    |   1    |   2    |   3    |
 |01234567|01234567|01234567|01234567|
 +-----------------------------------+
 |      Magic      |Version |FLA00000|
 +-----------------------------------+
 |      Length     |    Checksum     |
 +-----------------------------------+
 |          Sequence number          |
 +-----------------------------------+
 |             Payload               |
 +-----------------------------------+

 Magick          = (MSB) 0101101110111101 (LSB)
 Version         = 00000001  (1 dec)
 A               = 0
 F               = 1 if first packet
 L               = 1 if last packet
 Sequence number = random number incremented by 101 (dec) for each packet
 Length          = length of payload in number of bytes; max length = 64?
 Checksum        = least significant 16 bits of crc32 of packet with checksum = 0
 Payload         = some data (e.g. ip packet)


 ACK packet
 TODO: i think we do not need an ACK packet as TCP/IP is doing transmission
       control for us
 
 +--------+--------+--------+--------+
 |   0    |   1    |   2    |   3    |
 |01234567|01234567|01234567|01234567|
 +-----------------------------------+
 |      Magic      |Version |FLA00000|
 +-----------------------------------+
 |      Length     |    Checksum     |
 +-----------------------------------+
 |          Sequence number          |
 +-----------------------------------+

 Magick          = (MSB) 0101101110111101 (LSB)
 Version         = 00000001  (1 dec)
 A               = 1
 F               = 0
 L               = 0
 Sequence number = sequence number of received packet
 Length          = 0
 Checksum        = checksum of received packet


STEALTHY PROTOCOL
===============================================================================

 Receiving / sending a byte over air

 Receiving module

      +---------------+  on event: do not receive  +---------+
      |               |--------------------------->|         |
  +-->| ready to send |                            | waiting |
  |   |               |<---------------------------|         |
  |   +---------------+  on event: do receive      +---------+
  |          |
  |          |
  |          | on signal on air
  |          | => emit event: air busy
  |          V
  |   +---------------+
  |   | consume start |
  |   |      bit      |
  |   +---------------+
  |          |
  |          V
  |   +---------------+
  |   | receive data  |
  |   |     bits      |
  |   +---------------+
  |          |
  |          | put data byte into queue
  |          | => emit event: air not busy
  |          V
  |   +---------------+
  +---|    received   |
      +---------------+

  Sending module

      +------------------+  on event: air busy        +---------+
      |                  |--------------------------->|         |
  +-->| ready to receive |                            | waiting |
  |   |                  |<---------------------------|         |
  |   +------------------+  on event: air not busy    +---------+
  |          |
  |          |
  |          | data byte
  |          | => emit event: do not receive
  |          V
  |   +---------------+
  |   |   send data   |
  |   |     byte      |
  |   +---------------+
  |          |
  |          | => emit event: do receive
  |          V
  |   +---------------+
  +---|    done       |
      +---------------+

 Some notes:

 * If the sending module is sending a byte the receiving module should not
   receive it. Thus, sending and receiving module needs to be synchronized
   via the waiting state.
 * On the event "air busy" the sending module is not allowed to send any
   data.
 * On the event "do not receive" the receiving module is not allowed to
   receive any data.
 * After sending the data bits there should be enough time (due to the two
   zero bits) for the sender to take over the air to send data.





SENDING DATA
===============================================================================

TODO
  +-------------+
  | Application |
  +-------------+
        |
        |  send data via 'send'
        V
  +-------------+
  | IP packet   |  OS creates an IP packet
  +-------------+
        |
        |  IP packet is transmitted to the virtual device driver
        V
  +--------------------------+
  | virtual device driver    |
  | * create stealthy packet |
  |   with ip packet as      |
  |   payload                |
  +--------------------------+
        |
        | transmit each bit of stealthy packet to sound card
        V
  +------------+
  | sound card |
  +------------+
        |
        V
  +------------+
  | sender     |
  +------------+

FURTHER READING
===============================================================================

Develop a virtual device driver
https://www.kernel.org/doc/Documentation/networking/tuntap.txt

https://de.wikipedia.org/wiki/TUN/TAP

