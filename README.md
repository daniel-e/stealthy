# Stealthy
Stealthy is a small chat application for the console which uses ICMP echo requests (ping) for the communication with other clients. There are some advantages by using ICMP echo request.  First, firewalls which block TCP connections often allow ping messages to pass so that a communication is often possible even behind a firewall. Second, the communication is hidden. Ping packets are often ignored by system administrators. 

**All communication is encrypted**! Currently a hybrid approach is used where Blowfish is used to encrypt the messages (for fast encryption with small overhead) and RSA is used to encrypt the encryption key that is used by Blowfish.

## Installation

Before you can use stealthy you need some packages to compile successfully. When you're running Ubuntu you can install the packages as follows:

```sudo apt-get install build-essential libpcap-dev libssl-dev libncursesw5-dev```

Stealthy is written in the Rust programming language. Currently, there is not Rust package for Ubuntu available. You can install Rust in /opt/rust/ as follows:


```bash
# download rust
wget https://static.rust-lang.org/dist/rust-1.0.0-x86_64-unknown-linux-gnu.tar.gz

# install rust
tar xzf rust-1.0.0-x86_64-unknown-linux-gnu.tar.gz
cd rust-1.0.0-x86_64-unknown-linux-gnu/
./install.sh --prefix=/opt/rust

# export the search paths for the rust binary and rust libraries
export PATH=/opt/rust/bin/:$PATH
export LD_LIBRARY_PATH=/opt/rust/lib/:$LD_LIBRARY_PATH
```

Now, that you have installed all requirements you can compile stealthy as follows:



