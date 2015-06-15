# Build a release

This directory contains documentation and files required to build a binary release from the sources via Docker.

The initial step is to build an image which can later be used to build the release. Go into the directory which contains the file ```Dockerfile``` and execute the following command:

```bash
sudo docker build --rm -t buildgit/stealthy .
```

The image is based on Ubuntu 14.04.2.

After this step you can use this image as often as you want to build a release as follows:

```bash
sudo ~/docker/docker-latest run -t -v /tmp:/host buildgit/stealthy /tmp/build.sh
```

After successfull termination the binary file ```stealth``` is located in ```/tmp```.

