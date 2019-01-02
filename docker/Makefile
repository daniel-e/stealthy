all:
	docker build --no-cache -t stealthy/1 .

cache:
	docker build -t stealthy/1 .


run:
	docker run -t -i --rm --privileged -v ${HOME}:/host stealthy/1
