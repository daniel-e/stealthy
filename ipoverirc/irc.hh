#ifndef IRC_HH
#define IRC_HH

#include <string>
#include <functional>

void send_msg(std::string& data);
void irc_init(std::function<void(const std::string&)> f);

#endif




