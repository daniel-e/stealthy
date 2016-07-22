Stealthy enhancement proposal 

SEP     : 0
Title   : Chat via electro magnetic waves
Status  : Draft
Type    : Standard
Creates : 2016/07/22



STEALTHY PACKET TRANSMISSION OVER AIR
===============================================================================

* carrier frequency = 10MHz
* for each data byte: start bit + 8 data bits + 2 zero bits (11 bit)
* start bit is used to signal the beginning of the transmission of 8 data bits and to reduce effects caused by clock drifts
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

