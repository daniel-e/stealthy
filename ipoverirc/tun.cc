#include <sys/socket.h>
#include <linux/if.h>
#include <linux/if_tun.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <stdio.h>
#include <unistd.h>
#include <stdint.h>
#include <stdlib.h>

int tun_alloc(const char *dev)
{
	struct ifreq ifr;
	int fd, err;

	if( (fd = open("/dev/net/tun", O_RDWR)) < 0 )
		return -1;

	memset(&ifr, 0, sizeof(ifr));
	ifr.ifr_flags = IFF_TUN; 
	if(*dev)
		strncpy(ifr.ifr_name, dev, IFNAMSIZ);

	if((err = ioctl(fd, TUNSETIFF, (void *) &ifr)) < 0){
		close(fd);
		perror("ioctl");
		return -2;
	}

	return fd;
}

