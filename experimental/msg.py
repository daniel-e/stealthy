#!/usr/bin/env python

import math, struct, sys, time

# intel uses little endian
# gen.py | aplay -r 44100 -f S16_LE data.wav

rate = 44100
ampmax = 2**15 - 1
f = 1000

def f_sin(t, f, amax):
	return math.sin(t * 2 * math.pi * f) * amax

def play_sound(f, ampmax, td):
	s = time.time()
	samples = int(td * rate)  # number of samples
	for i in xrange(samples):
		t = 1.0 * i / rate
		d = int(f_sin(t, f, ampmax))
		sys.stdout.write(struct.pack("h", d)) # short (standard size = 2 byte)
	x = time.time()
	w = td - (x - s)
	if w > 0:
		time.sleep(w)

def send_signal(value):
	if value == 1:
		print >> sys.stderr, "1"
		play_sound(1300, ampmax, 0.1)
	else:
		print >> sys.stderr, "0"
		play_sound(1000, ampmax, 0.1)

# sends byte: first most significant bit
def send_byte(b):
	b = ord(b)
	c = 128
	s = time.time()
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
	x = time.time()

def send_msg(msg):
	for m in msg:
		send_byte(m)

send_msg("Hello")

