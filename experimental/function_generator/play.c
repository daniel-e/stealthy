#include <linux/soundcard.h>
#include <stdio.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>
#include <math.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/time.h>

int rate = 8000;
int bits = 8;
double tsignal = 0.001;
unsigned char* signal_1a;
unsigned char* signal_1b;
unsigned char* signal_0;
int freq = 1000;  // for sinus
int amax = 255;

// http://stackoverflow.com/questions/5890499/pcm-audio-amplitude-values
// 128 is silence

void send_signal(int fd, unsigned char value) {

	static int cnt = 0;

	if (value) {
		printf("1");
		if (++cnt % 2) {
			write(fd, signal_1a, (int) (tsignal * rate));
		} else {
			write(fd, signal_1b, (int) (tsignal * rate));
		}
	} else {
		printf("0");
		write(fd, signal_0, (int) (tsignal * rate));
	}
}

void send_byte(int fd, unsigned char x) {

	struct timeval tm1;
	struct timeval tm2;
	unsigned long long t;

	unsigned char c = 128;

	gettimeofday(&tm1, NULL);

	send_signal(fd, 1);
	for (; c; c = c >> 1) {
		if (x & c) {
			send_signal(fd, 1);
		} else {
			send_signal(fd, 0);
		}
	}

	send_signal(fd, 0);
	send_signal(fd, 0);
	printf("\n");

#if 0
	while (1) {
		gettimeofday(&tm2, NULL);
		t = 1000 * (tm2.tv_sec - tm1.tv_sec) + (tm2.tv_usec - tm1.tv_usec) / 1000;
		if (t >= tsignal * 1000 * 21) {
			break;
		}
		usleep(tsignal * 1000 * 21 - t);
	}
#endif

#if 0
	if (ioctl(fd, SOUND_PCM_SYNC, 0) == -1) {
		perror("SOUND_PCM_SYNC");
	}
#endif
}


void init_sin(char* buf, char* buf2, int n) {

	int i;

	for (i = 0; i < n; ++i) {
		double t = (double) i / rate;
		double v = (1.0 + sin(t * 2.0 * 3.14159 * freq)) * amax / 2;
		buf[i] = (unsigned char) v;
	}
	memcpy(buf2, buf, n);
}

void init_rec(char* buf, char* buf2, int n) {

	memset(buf, 255, n);
	memset(buf2, 0, n);

#if 0
	memset(buf, 128, n);
	buf[0] = 255;
	buf[1] = 0;
#endif

#if 0
	for (int i = 0; i < n; ++i) {
		buf[i] = 0;
		if (i & 1) {
			buf[i] = 255;
		}
	}
	//memset(buf, high_val, n);
#endif
}

void init_buffers(int argc, char** argv) {

	int n = (int) (tsignal * rate);
	signal_1a = (unsigned char*) malloc(n);
	signal_1b = (unsigned char*) malloc(n);
	signal_0 = (unsigned char*) malloc(n);
	memset(signal_0, 128, n);
	if (argc > 1) {
		if (!strcmp(argv[1], "sin")) {
			printf("sin\n");
			init_sin(signal_1a, signal_1b, n);
		} else if (!strcmp(argv[1], "rec")) {
			printf("rec\n");
			init_rec(signal_1a, signal_1b, n);
		} else {
			printf("unknown parameter\n");
			exit(1);
		}
	} else {
		printf("sin\n");
		init_sin(signal_1a, signal_1b, n);
	}
}

int main(int argc, char** argv) {


	init_buffers(argc, argv);

	int fd = open("/dev/audio", O_RDWR);
	if (fd < 0) {
		perror("open");
		return 1;
	}
#if 0
	unsigned char* b = malloc(rate * 10);
	memset(b, 128, rate * 10);
	for (int i = 0; i < rate * 10; i += 100) {
		b[i] = 0;
	}
	write(fd, b, rate * 10);
	sleep(10);
	return 1;
#endif
	while (1) {
		unsigned char x = 0b01111001;
		printf("sending... \n");
		send_byte(fd, x);
	}
}

