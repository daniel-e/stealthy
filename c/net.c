#include <netinet/tcp.h>  // struct tcphdr
#include <sys/socket.h>
#include <arpa/inet.h>
#include <unistd.h>       // close
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <pcap.h>
#include <pthread.h>

#include "net.h"

#define SIZE_ETHERNET    14
#define MAGIC 0xa387

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
		perror("socket() error");
		return ret;
	}

	struct sockaddr_in s;
	s.sin_family = AF_INET;
	s.sin_addr.s_addr = inet_addr(dstip);

	if (sendto(sd, packet, sizeof(struct icmp) + size, 0, (struct sockaddr*) &s, sizeof(s)) < 0) {
		perror("sendto() error");
		return ret;
	}
	close(sd);

	return 0;
}

pcap_t* setup_pcap(const char* dev, const char* filter)
{
	char errbuf[PCAP_ERRBUF_SIZE];
	pcap_t* handle = pcap_open_live(dev, PCAP_ERRBUF_SIZE, 1, 1000, errbuf);
	if (!handle) {
		fprintf(stderr, "setup_pcap(): could not open device %s\n", dev);
		return 0;
	}

	struct bpf_program bpf;
	bpf_u_int32 mask;
	bpf_u_int32 net;
	if (pcap_lookupnet(dev, &net, &mask, errbuf) == -1) {
		fprintf(stderr, "Can't get netmask for device %s\n", dev);
		net = 0;
		mask = 0;
	}

	if (pcap_compile(handle, &bpf, filter, 0, net) == -1) {
		fprintf(stderr, "could not set filter\n");
		return 0;
	}

	if (pcap_setfilter(handle, &bpf) == -1) {
		fprintf(stderr, "Couldn't install filter %s: %s\n", filter, pcap_geterr(handle));
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

void got_packet(u_char* args, const struct pcap_pkthdr* h, const u_char* packet)
{
	char buf[128];

	if (!h->len || h->len < SIZE_ETHERNET + 20) return;

	packet += SIZE_ETHERNET;

	u_int16_t iphdrlen = *packet & 0xf;       // little endian
	u_int8_t  proto    = *(packet + 9);       // protocol (should be 1)
	u_int16_t iplen    = ntohs(*(u_int16_t*)(packet + 2));
	u_int32_t srcip    = *(u_int32_t*)(packet + 12);

	inet_ntop(AF_INET, &srcip, buf, sizeof(buf));

	if (iplen < iphdrlen * 4) return;
	if (proto != 1) return;
	if (h->len < SIZE_ETHERNET + iphdrlen * 4 + sizeof(struct icmp)) return;

	packet += iphdrlen * 4;
	struct icmp* i = (struct icmp*) packet;

	if (i->type != 0 && i->type != 8) return; // no ping, no poing
	if (ntohs(i->id) != MAGIC) return;

	int datalen = iplen - iphdrlen * 4 - sizeof(struct icmp);
	int type    = (i->type == 0 ? PONG : PING);
	packet += sizeof(struct icmp);

	if (iphdrlen * 4 + sizeof(struct icmp) > iplen) return;
	if (h->len < SIZE_ETHERNET + iphdrlen * 4 + sizeof(struct icmp) + datalen) return;

	struct arguments* a = (struct arguments*) args;
	a->cb(a->target, (const char*) packet, datalen, type, buf);
}

static void* do_callback(void* args)
{
	struct arguments* a = (struct arguments*) args;
	pcap_loop(a->handle, -1, got_packet, (u_char*) a);
	pcap_close(a->handle);
	free(a);
	return 0;
}

int recv_callback(void* target, const char* dev, callback cb) {

	pcap_t* handle = setup_pcap(dev, "icmp");
	if (handle) {
		pthread_t t;
		struct arguments* args = (struct arguments*) malloc(sizeof(struct arguments));
		args->handle = handle;
		args->cb = cb;
		args->target = target;
		int r = pthread_create(&t, NULL, &do_callback, (void*) args);
		if (r != 0) {
			fprintf(stderr, "could not create thread");
			return -1;
		}
	} else return -1;
	return 0;
}

