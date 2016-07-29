#include <linux/soundcard.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdio.h>

// http://www.4front-tech.com/pguide/audio.html
// Value of 0 represents the minimum level and 255 the maximum. The neutral level is 128 (0x80).

int main(int argc, char** argv) {

	unsigned char buf[8000];

	int fd = open("/dev/audio", O_RDWR);
	memset(buf, 0, sizeof(buf));

	// default for /dev/audioi: 8000 Hz, 8 bit, unsigned, mono
	while (1) {
		printf("%d\n", (int) write(fd, buf, sizeof(buf)));
	}
}

