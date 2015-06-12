# Stealthy
Stealthy is a small chat application for the console which uses ICMP echo requests (ping) for the communication with other clients. There are some advantages when ICMP echo request are used.  First, if a firewall is configured to block TCP connections there's a pretty good chance that this firewall is not configured to reject ping messages so that a communication is often possible even when you're behind a firewall. Second, the communication is hidden. Ping packets are often ignored by system administrators and will not be investigated. Thus, when using stealthy the fact that a communication is happening will often not be recognized.

**All communication is encrypted**! Currently you can choose between a pure symmetric encryption with Blowfish or an hybrid approach where Blowfish is used to encrypt the messages (for fast encryption with small overhead) and RSA is used to encrypt the encryption key that is used for the Blowfish encryption.

## Running stealthy

Stealthy always required two command line arguments:
* `-i` specifies the network interface which is used to listen for incoming messages
* `-d` specifies the IP address of the receiver


Further, stealthy can be used in two encryption modes: symmetric encryption and hybrid encryption.

**Symmetric encryption** is choosen with the command line argument `-e` followed by a 128 bit encryption key in hexadecimal (i.e. 32 characters in the range 0..9 and a..f). If -e is not given the default key `11111111111111111111111111111111` is used. Although the messages are not transmitted in plaintext when the default key is used everyone who is in the possession of that key could decrypt your messages. Thus, **use the default key with caution**!

Examples to use stealthy with symmetric encryption:
```bash
# stealthy with default encryption key
sudo ./stealthy -i eth0 -d 1.2.3.4

# stealthy with a use defined encryption key
sudo ./stealthy -i eth0 -d 1.2.3.4 -e a1515134c543aafca4796a256839a6b2
```

*btw: you could use to following command to create good keys: `cat /dev/urandom | xxd -p -l 16`*

**Asymmetric encryption**

There is one drawback that comes with the symmetric encryption mode. Both chat clients have to use the same key so you have to exchange the key with your chat partner before you can chat. Exchanging the key securely is often difficult or even not possible. Thus, stealthy also supports a hybrid encryption.

Hybrid encryption is actived with the command line arguments `-r` and `-p`. Both arguments are required to enable the hybrid encryption. With -r (r for receiver) you can specify the name of the file which contains the public key of the receiver. The public key is used for encrypting a message and only the client which is in possession of the corresponding private key can decrypt the message. With -p (p for private) you can specify the name of the file which contains your private key which is used to decrypt the messages that have been encrypted with your public key.

Here is an example: let's assume Alice and Bob want to communicate via stealthy. Alice creates a public key `pubA` and a private key `privA`. Bob creates a public key `pubB` and a private key `privB`. Alice sends her public key `pubA` to Bob and Bob sends his public key `pubB` to Alice. Now, to communicate Alice and Bob start stealthy as follows:

```bash
# Alice
sudo ./stealthy -i eth0 -d 1.2.3.4 -r pubB -p privA
# Bob
sudo ./stealthy -i eth0 -d 2.4.1.2 -r pubA -p privB
```



## Installation

### Binary

For Linux you can download a binary which has been compiled for Linux Mint 17.1 and should work for Ubuntu as well.

[https://github.com/daniel-e/icmpmessaging-rs/releases/download/v0.0.1/stealthy](https://github.com/daniel-e/icmpmessaging-rs/releases/download/v0.0.1/stealthy)

### From Sources

Before you can use stealthy you need some packages to be able to compile the sources successfully. When you're running Ubuntu you should install the following packages (if not already installed):

```
sudo apt-get install build-essential libpcap-dev libssl-dev libncursesw5-dev
```

Stealthy is written in the Rust programming language. Currently, there is not Rust package for Ubuntu available so you have to install Rust manually. You can install Rust in /opt/rust/ as follows:


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

```bash
# checkout the sources
git clone https://github.com/daniel-e/icmpmessaging-rs.git
# build the sources
cd icmpmessaging-rs
cargo build
```

You can now start stealthy with the command ```sudo ./target/debug/icmpmessaging```.
