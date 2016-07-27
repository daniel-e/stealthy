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

int rate = 8000;
int bits = 8;
double tsignal = 0.001;
char* signal_1;
char* signal_0;
int freq = 1000;  // for sinus
int amax = 127;

void send_signal(int fd, unsigned char value) {

	if (value) {
		printf("1");
		write(fd, signal_1, (int) (tsignal * rate));
	} else {
		printf("0");
		write(fd, signal_0, (int) (tsignal * rate));
	}
}

void send_byte(int fd, unsigned char x) {

	int i;
	unsigned char c = 128;
	int status;

	send_signal(fd, 1);

	for (i = 0; i < 8; ++i) {
		if (x & c) {
			send_signal(fd, 1);
		} else {
			send_signal(fd, 0);
		}
		c = c >> 1;
	}

	send_signal(fd, 0);
	send_signal(fd, 0);
	printf("\n");

	status = ioctl(fd, SOUND_PCM_SYNC, 0);
	if (status == -1) {
		perror("SOUND_PCM_SYNC");
	}
}


void init_sin(char* buf, int n) {

	int i;

	for (i = 0; i < n; ++i) {
		double t = (double) i / rate;
		double v = sin(t * 2.0 * 3.14159 * freq) * amax;
		signal_1[i] = (char) v;
	}
}

void init_rec(char* buf, int n) {

	memset(buf, 127, n);
}


int main(int argc, char** argv) {

	// initialize buffers
	int n = (int) (tsignal * rate);
	signal_1 = (char*) malloc(n);
	signal_0 = (char*) malloc(n);
	memset(signal_0, 0, n);
	if (argc > 1) {
		if (!strcmp(argv[1], "sin")) {
			printf("sin\n");
			init_sin(signal_1, n);
		} else if (!strcmp(argv[1], "rec")) {
			printf("rec\n");
			init_rec(signal_1, n);
		} else {
			printf("unknown parameter\n");
			return 1;
		}
	} else {
		printf("sin\n");
		init_sin(signal_1, n);
	}

	int fd = open("/dev/audio", O_RDWR);
	if (fd < 0) {
		perror("open");
		return 1;
	}

	while (1) {
		unsigned char x = 0b01110101;
		printf("sending... \n");
		send_byte(fd, x);
	}
}

