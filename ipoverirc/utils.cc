#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>

#include "utils.hh"

std::string random_string(int n) 
{
	uint32_t x;
	std::string data = "abcdefghijklmnopqrstuvwxyz";
	std::string r;
	int fd = open("/dev/urandom", O_RDONLY);
	if (fd == -1) {
		perror("open /dev/urandom");
		exit(1);
	}
	for (int i = 0; i < n; ++i) {
		if (read(fd, &x, 4) == -1) {
			perror("read /dev/urandom");
			exit(1);
		}
		r = r + data[x % data.size()];
	}
	close(fd);
	return r;
}

