FLAGS=-std=c++11

all: tun.o main.o irc.o utils.o
	g++ main.o tun.o irc.o utils.o -o main -lboost_system -lboost_thread -lpthread

%.o: %.cc %.hh
	g++ $(FLAGS) -c $<

clean:
	rm -f main *.o
