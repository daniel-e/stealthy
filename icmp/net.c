#include <netinet/tcp.h>  // struct tcphdr
#include <sys/socket.h>
#include <arpa/inet.h>
#include <unistd.h>       // close
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <pcap.h>
#include <pthread.h>
#include <stdio.h>

#include "net.h"

#define SIZE_ETHERNET    14
#define MAGIC 0xa387

#undef DEBUG_NETC

// http://tools.ietf.org/html/rfc793
// http://tools.ietf.org/html/rfc1071
// http://locklessinc.com/articles/tcp_checksum/
u_int16_t chksum(const char* buffer, int size)
{
	u_int32_t sum = 0;
	int i;

	for (i = 0; i < size - 1; i += 2) {
		sum += *(unsigned short*) &buffer[i];
	}
	if (size & 1) sum += (unsigned) (unsigned char) buffer[size - 1];
	while (sum >> 16) sum = (sum & 0xffff) + (sum >> 16);
	return ~sum;
}

struct icmp
{
	u_int8_t type;
	u_int8_t code;
	u_int16_t sum;
	u_int16_t id;
	u_int16_t seq;
};


int send_icmp(const char* dstip, const char* buf, u_int16_t size)
{
	int ret = -1;
	char*     packet = (char*) malloc(sizeof(struct icmp) + size);

	if (size > (1 << 14)) {
		perror("packet too large.");
		return ret;
	}

	if (!packet) {
		perror("malloc()");
		return ret;
	}

	u_int16_t seq = rand();

	// copy data into icmp packet
	memcpy(packet + sizeof(struct icmp), buf, size);

	struct icmp* i = (struct icmp*) packet;
	i->type = 8; // echo request
	i->code = 1;
	i->sum = 0;
	i->id = htons(MAGIC);
	i->seq = htons(seq);
	i->sum = chksum(packet, sizeof(struct icmp) + size);

	// open socket and send packet
	int sd = socket(PF_INET, SOCK_RAW, IPPROTO_ICMP);
	if (sd < 0) {
		return ret;
	}

	struct sockaddr_in s;
	s.sin_family = AF_INET;
	s.sin_addr.s_addr = inet_addr(dstip);

	if (sendto(sd, packet, sizeof(struct icmp) + size, 0, (struct sockaddr*) &s, sizeof(s)) < 0) {
		return ret;
	}
	close(sd);

	return 0;
}

pcap_t* setup_pcap(const char* dev, const char* filter)
{
    #define MXSIZ 32000
	char errbuf[MXSIZ];
	/*char errbuf[PCAP_ERRBUF_SIZE];*/
	// Timeout should be small. Otherwise receiving packets could be delayed.
	// https://stackoverflow.com/a/30203212/4339066
	int timeout = 1; // ms
	pcap_t* handle = pcap_open_live(dev, MXSIZ, 1, timeout, errbuf);
	if (!handle) {
		return 0;
	}

	struct bpf_program bpf;
	bpf_u_int32 mask;
	bpf_u_int32 net;
	if (pcap_lookupnet(dev, &net, &mask, errbuf) == -1) {
		net = 0;
		mask = 0;
	}

	if (pcap_compile(handle, &bpf, filter, 0, net) == -1) {
		return 0;
	}

	if (pcap_setfilter(handle, &bpf) == -1) {
		return 0;
	}
	return handle;
}

struct arguments
{
	pcap_t*  handle;
	callback cb;
	void*    target;
};

// TODO refactor: check_ip_packet and got_packet
int check_ip_packet(const struct pcap_pkthdr* h, const u_char* packet)
{
	// at least 20 bytes are required
	if (h->len < 20) {
		return -1;
	}

	u_int16_t iphdrlen = *packet & 0xf;       // little endian
	u_int8_t  proto    = *(packet + 9);       // protocol (should be 1)
	u_int16_t iplen    = ntohs(*(u_int16_t*)(packet + 2));

	// check length of packet
	if (iplen < iphdrlen * 4) {
		return -1;
	}
	// check protocol
	if (proto != 1) {
		return -1;
	}

	if (h->len < iphdrlen * 4 + sizeof(struct icmp)) {
		return -1;
	}

	packet += iphdrlen * 4;
	struct icmp* i = (struct icmp*) packet;

	if (i->type != 0 && i->type != 8) { // check that ping or pong
		return -1;
	}

	if (ntohs(i->id) != MAGIC) {
		return -1;
	}

	int datalen = iplen - iphdrlen * 4 - sizeof(struct icmp);
	int type    = (i->type == 0 ? PONG : PING);
	packet += sizeof(struct icmp);

	if (iphdrlen * 4 + sizeof(struct icmp) > iplen) {
		return -1;
	}

	if (h->len < iphdrlen * 4 + sizeof(struct icmp) + datalen) {
		return -1;
	}

	return 0;
}

void got_packet(u_char* args, const struct pcap_pkthdr* h, const u_char* packet)
{
	struct    arguments* a = (struct arguments*) args;
	char      buf[128];
	u_int32_t size_ethernet = SIZE_ETHERNET;

#if DEBUG_NETC
    FILE* f;
    int c;
    f = fopen("/tmp/icmp.log", "a");
#endif

	// check if this is already an IP packet; otherwise it could be an etheret
	// frame
	if (check_ip_packet(h, packet) == 0) {
		size_ethernet = 0;
	}

	// at least 20 bytes are required
	if (h->len < 20) {
		a->cb(a->target, 0, 0, INVALID_LENGTH, 0);
		return;
	}

	packet += size_ethernet;

	u_int16_t iphdrlen = *packet & 0xf;       // little endian
	u_int8_t  proto    = *(packet + 9);       // protocol (should be 1)
	u_int16_t iplen    = ntohs(*(u_int16_t*)(packet + 2));
	u_int32_t srcip    = *(u_int32_t*)(packet + 12);

	inet_ntop(AF_INET, &srcip, buf, sizeof(buf));

	// check length of packet
	if (iplen < iphdrlen * 4) {
		a->cb(a->target, 0, 0, INVALID_IP_LENGTH, 0);
#if DEBUG_NETC
		fprintf(f, "Packet has invalid length.\n");
		fflush(f);
		fclose(f);
#endif
		return;
	}
	// check protocol
	if (proto != 1) {
		a->cb(a->target, 0, 0, INVALID_PROTOCOL, 0);
		return;
	}

	if (h->len < size_ethernet + iphdrlen * 4 + sizeof(struct icmp)) {
		a->cb(a->target, 0, 0, INVALID, 0);
#if DEBUG_NETC
		fprintf(f, "Packet has invalid length (2).\n");
		fflush(f);
		fclose(f);
#endif
		return;
	}

	packet += iphdrlen * 4;
	struct icmp* i = (struct icmp*) packet;

	// check that ping or pong
	if (i->type != 0 && i->type != 8) {
		a->cb(a->target, 0, 0, INVALID, 0);
		return;
	}

	if (ntohs(i->id) != MAGIC) {
		a->cb(a->target, 0, 0, INVALID, 0);
		return;
	}

	int datalen = iplen - iphdrlen * 4 - sizeof(struct icmp);
	int type    = (i->type == 0 ? PONG : PING);
	packet += sizeof(struct icmp);

	if (iphdrlen * 4 + sizeof(struct icmp) > iplen) {
		a->cb(a->target, 0, 0, INVALID, 0);
#if DEBUG_NETC
		fprintf(f, "Packet has invalid length (3).\n");
		fflush(f);
		fclose(f);
#endif
		return;
	}

	if (h->len < size_ethernet + iphdrlen * 4 + sizeof(struct icmp) + datalen) {
		a->cb(a->target, 0, 0, INVALID, 0);
#if DEBUG_NETC
		fprintf(f, "Packet has invalid length (4).\n");
		fflush(f);
		fclose(f);
#endif
		return;
	}

#if DEBUG_NETC
		fprintf(f, "Calling callback. type = %d, datalen = %d.\n", type, datalen);
		fflush(f);
		for (c = 0; c < datalen; c++) {
		    fprintf(f, "%x ", packet[c]);
		}
		fprintf(f, "\n");
		fflush(f);
		fclose(f);
#endif
	a->cb(a->target, (const char*) packet, datalen, type, buf);
}

static void* worker_thread(void* args)
{
	struct arguments* a = (struct arguments*) args;
	// processes packets from a live capture; does not return
	// https://linux.die.net/man/3/pcap_loop
	pcap_loop(a->handle, -1, got_packet, (u_char*) a);
	pcap_close(a->handle);
	free(a);
	return 0;
}

int recv_callback(void* target, const char* dev, callback cb) {

	pcap_t* handle = setup_pcap(dev, "icmp && icmp[icmptype] = 8");
	if (handle) {
		pthread_t t;
		struct arguments* args = (struct arguments*) malloc(sizeof(struct arguments));
		args->handle = handle;
		args->cb = cb;
		args->target = target;
		int r = pthread_create(&t, NULL, &worker_thread, (void*) args);
		if (r != 0) {
			return -1;
		}
	} else return -1;
	return 0;
}
