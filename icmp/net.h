#ifndef NET_HH
#define NET_HH

enum {
	PING,
	PONG,
	INVALID_LENGTH,
	INVALID_IP_LENGTH,
	INVALID_PROTOCOL,
	INVALID
};

typedef const unsigned char* u8_ptr;
typedef void(*callback)(void*, const char* buf, u_int32_t len, u_int32_t type, u8_ptr srcip);

// returns 0 on success
int         send_icmp(const char* dstip, const char* buf, u_int16_t size);
// returns 0 on success
int         recv_callback(void* target, const char* dev, callback);

#endif
