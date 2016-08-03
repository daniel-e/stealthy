#include <unistd.h>
#include <stdlib.h>
#include <iostream>
#include <string>

#include "base64.h"
#include "irc.hh"
#include "tun.hh"

int fd_dev; // file descriptor for virtual network device

// Callback function that is called if an IP packet has been received
// from the IRC channel.
void cb(const std::string& msg)
{
	std::cout << "got IP packet <" << msg.substr(0, 30) << "...>" << std::endl;

	std::string out;
	Base64::Decode(msg, &out);
	if (write(fd_dev, out.data(), out.size()) != out.size()) {
		perror("write to network device");
	}
}

int main(int argc, char** argv) 
{
	if (argc < 2) {
		std::cerr << "usage: " << argv[0] << " devicename" << std::endl;
		return -1;
	}

	// Connect to IRC.
	std::cout << "connecting to IRC ..." << std::endl;
	irc_init(cb);

	char buf[2048];
	int  n;

	std::cout << "creating network device " << argv[1] << " ..." << std::endl;
	// Create the virtual network device.
	if ((fd_dev = tun_alloc(argv[1])) < 0) {
		perror("tun_alloc");
		return 1;
	}

	std::cout << "running" << std::endl;
	while (1) {
		// read data from virtual network device
		n = read(fd_dev, buf, sizeof(buf));
		if (n == -1) {
			perror("read");
			return 1;
		} else if (n == 0) {
			printf("end of file\n");
			return 0;
		} else {
			std::string data(buf, n);
			std::string b;
			Base64::Encode(data, &b);
			// send base64 encoded IP packet to IRC
			send_msg(b);
		}
	}
}

