#!/usr/bin/env python

import math, struct, sys, time

# intel uses little endian
# gen.py | aplay -r 44100 -f S16_LE data.wav
f = open("/tmp/aaaaa", "w")
print f.write('abc')


rate = 44100
rate = 8000
ampmax = 2**15 - 1
ampmax = 2**8 - 1
f = 1000
tsignal = 0.1
stopbits = 2

audio = open("/dev/audio", "w")

def f_sin(t, f, amax):
	return math.sin(t * 2 * math.pi * f) * amax


def play_sound(f, ampmax, td):
	samples = int(td * rate)  # number of samples
	print samples
	for i in xrange(samples):
		t = 1.0 * i / rate
		d = int(f_sin(t, f, ampmax))
		d = struct.pack("h", d)
		audio.write(d)
		audio.flush()

def send_signal(value):
	w1 = time.time()
	if value == 1:
		print >> sys.stderr, "1"
		play_sound(f, ampmax, tsignal)
	else:
		print >> sys.stderr, "0"
		play_sound(f, 0, tsignal)
	w2 = time.time()
	print "XX", w2 - w1

# sends byte: first most significant bit
def send_byte(b):
	b = ord(b)
	c = 128
	w1 = time.time()
	send_signal(1)
	for i in xrange(8):
		if b & c > 0:
			send_signal(1)
			b -= c
		else:
			send_signal(0)
		c = c / 2
	#send_signal(0)
	#send_signal(0)
	while True:
		w2 = time.time()
		d = w2 - w1
		if d >= tsignal * (9 + stopbits):
			print "done", d, tsignal * (9 + stopbits)
			break
		print "waiting"
		time.sleep(tsignal * (9 + stopbits) - d)
	w2 = time.time()
	print "OK", w2 - w1

#w1 = time.time()
#f = open("/dev/urandom")
#d = f.read(64000)
#audio.write(d)
#w2 = time.time()
#print w2 - w1
#time.sleep(20)
#sys.exit(0)

while True:
	print >> sys.stderr, "next byte ..."
	x = chr(int('01010101', 2))
	x = chr(int('00000001', 2))
	send_byte(x)

	print "DONE"
	time.sleep(10)
	
#	sys.exit(0)

def send_msg(msg):
	for m in msg:
		send_byte(m)

send_msg("A")

