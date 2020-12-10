use std::net::Ipv4Addr;

pub struct IpAddresses {
    ips: Vec<Ipv4Addr>
}

impl IpAddresses {
    pub fn from_comma_list(s: &str) -> IpAddresses {
        IpAddresses {
            ips: s.split(",")
                .map(|x| String::from(x).trim().to_string())
                .filter(|x| x.len() > 0)
                .map(|x| x.parse().expect("Found invalid IP address."))
                .collect()
        }
    }

    pub fn as_strings(&self) -> Vec<String> {
        self.ips.iter().map(|x| x.to_string()).collect()
    }

    pub fn set_ip(&mut self, ip: String) {
        match ip.parse() {
            Err(_) => {},
            Ok(ip) => { self.ips = vec![ip]; }
        }
    }
}


