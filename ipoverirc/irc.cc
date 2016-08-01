#include <string>

#include <string>
#include <iostream>
#include <istream>

#include <boost/asio.hpp>
#include <boost/thread.hpp>
using boost::asio::ip::tcp;

#include "utils.hh"

std::string generate_nick() 
{
	return std::string("STEALTHY") + random_string(30);
}

std::string generate_user()
{
	return std::string("USER") + random_string(30);
}


boost::asio::io_service io_service;

std::string nick;
std::string user;

const std::string ircsrv = "irc.swiftirc.net";
const std::string port = "7000";
const std::string channel = "stealthyipoverirc";

tcp::socket* sock_;
boost::asio::streambuf buf;
std::string data_in;

std::function<void(const std::string&)> callback_;

std::string generate_nick();
std::string generate_user();

void irc_connect() 
{
	sock_ = new tcp::socket(io_service);
	tcp::resolver resolver(io_service);
	tcp::resolver::query query(ircsrv, port);
	auto endpoint_iterator = resolver.resolve(query);
	boost::asio::connect(*sock_, endpoint_iterator);
}

void send_line(const std::string& data) 
{
	std::cout << "sending <" << data.substr(0, 30) << "...>" << std::endl;
	boost::asio::write(*sock_, boost::asio::buffer(data.data(), data.size()));
	boost::asio::write(*sock_, boost::asio::buffer("\n", 1));
	// flood detection protection
	sleep(1);
}

void parse_buffer()
{
	while (1) {
		std::string::size_type n = data_in.find("\n");
		if (n == std::string::npos) {
			break;
		}
		std::string msg = data_in.substr(0, n);
		while (msg.size() && msg[msg.size() - 1] == '\r') {
			msg.erase(msg.size() - 1, 1);
		}
		data_in.erase(0, n + 1);
		//std::cout << "GOT MSG <" << msg << ">" << std::endl;
		if (msg.substr(0, 5) == "PING ") {
			msg.erase(0, 5);
			std::string pong_msg = std::string("PONG ") + msg;
			send_line(pong_msg);
		} else {
			std::string q = std::string("PRIVMSG #") + channel + " :";
			std::string::size_type n = msg.find(q);
			if (n != std::string::npos) {
				msg = msg.substr(n + 9 + channel.size() + 2);
				//std::cout << "IP PACKET <" << msg << ">" << std::endl;
				if (callback_) {
					callback_(msg);
				}
			}
		}
	} 
}

void read_data(const boost::system::error_code& error, std::size_t bytes_transferred)
{
	std::istream is(&buf);
	while (bytes_transferred) {
		char buffer[4096];
		size_t n = std::min(bytes_transferred, sizeof(buffer));
		is.read(buffer, n);
		data_in.append(buffer, n);
		bytes_transferred -= n;
	}

	parse_buffer();
	boost::asio::async_read_until(*sock_, buf, '\n', &read_data);
}

void send_msg(std::string& data)
{
	std::string msg = std::string("PRIVMSG #") + channel + " :" + data;
	send_line(msg);
}

void irc_init(std::function<void(const std::string&)> f)
{
	callback_ = f;
	irc_connect();
	boost::asio::async_read_until(*sock_, buf, '\n', read_data);
	static boost::thread service_thread(boost::bind(&boost::asio::io_service::run, &io_service));

	sleep(1);
	nick = generate_nick();
	std::string nick_msg = std::string("NICK ") + nick;
	send_line(nick_msg);

	sleep(1);
	user = generate_user();
	std::string user_msg = std::string("USER ") + user + " 0 * :stealthy ip";
	send_line(user_msg);

	sleep(1);
	std::string join_msg = std::string("JOIN ") + ":#" + channel;
	send_line(join_msg);
}

