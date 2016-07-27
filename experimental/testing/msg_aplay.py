#!/usr/bin/env python

import math, struct, sys, time

# intel uses little endian
# gen.py | aplay -r 44100 -f S16_LE data.wav

rate = 44100
rate = 8000
ampmax = 2**15 - 1
ampmax = 2**8 - 1
f = 1000

#audio = open("/dev/audio", "w")

def f_sin(t, f, amax):
	return math.sin(t * 2 * math.pi * f) * amax


def play_sound(f, ampmax, td):
	samples = int(td * rate)  # number of samples
	for i in xrange(samples):
		t = 1.0 * i / rate
		d = int(f_sin(t, f, ampmax))
		d = struct.pack("h", d)
		#audio.write(d)
		#audio.write(d)
		sys.stdout.write(d) # short (standard size = 2 byte)

def send_signal(value):
	if value == 1:
		print >> sys.stderr, "1"
		play_sound(f, ampmax, 0.1)
	else:
		print >> sys.stderr, "0"
		play_sound(f, 0, 0.1)

# sends byte: first most significant bit
def send_byte(b):
	b = ord(b)
	c = 128
	send_signal(1)
	for i in xrange(8):
		if b & c > 0:
			send_signal(1)
			b -= c
		else:
			send_signal(0)
		c = c / 2
	send_signal(0)
	send_signal(0)

def send_msg(msg):
	for m in msg:
		send_byte(m)

send_msg("A")

