#!/usr/bin/env python

import math, struct, sys

# intel uses little endian
# gen.py | aplay -r 44100 -f S16_LE data.wav

rate = 44100
ampmax = 2**15 - 1
f = int(sys.argv[1])

def f_sin(t, f, amax):
	return math.sin(t * 2 * math.pi * f) * amax

def f_rec(t, f, amax):
	t0 = 1.0 / f
	if t % t0 < t0 / 2:
		return ampmax
	else:
		return 0.0 - ampmax

i = 0
while True:
	t = 1.0 * i / rate
	d = int(f_sin(t, f, ampmax))
	#d = int(f_rec(t, f, ampmax))
	sys.stdout.write(struct.pack("h", d)) # short (standard size = 2 byte)
	i += 1
