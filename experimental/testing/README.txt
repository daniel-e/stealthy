intel uses little endian
gen.py | aplay -r 44100 -f S16_LE data.wav

padsp ./msg.py

./msg_aplay.py | aplay

