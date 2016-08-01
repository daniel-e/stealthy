FLAGS=-std=c++11

all: tun.o ipoverirc.o irc.o utils.o
	g++ $? -o ipoverirc -lboost_system -lboost_thread -lpthread

%.o: %.cc
	g++ $(FLAGS) -c $<

clean:
	rm -f ipoverirc *.o
