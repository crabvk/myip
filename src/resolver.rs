use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;

pub const OPENDNS_IPS_V4: &[IpAddr] = &[
    IpAddr::V4(Ipv4Addr::new(208, 67, 222, 222)),
    IpAddr::V4(Ipv4Addr::new(208, 67, 220, 220)),
    IpAddr::V4(Ipv4Addr::new(208, 67, 222, 220)),
    IpAddr::V4(Ipv4Addr::new(208, 67, 220, 222)),
];

pub const OPENDNS_IPS_V6: &[IpAddr] = &[
    IpAddr::V6(Ipv6Addr::new(0x2620, 0x119, 0x35, 0, 0, 0, 0, 0x35)),
    IpAddr::V6(Ipv6Addr::new(0x2620, 0x119, 0x53, 0, 0, 0, 0, 0x53)),
];

pub enum IpAddrKind {
    V4,
    V6,
}

pub fn dig(ip_kind: IpAddrKind) -> io::Result<IpAddr> {
    let mut name_servers = NameServerConfigGroup::new();
    let (ips, ip_strategy) = match ip_kind {
        IpAddrKind::V4 => (OPENDNS_IPS_V4, LookupIpStrategy::Ipv4Only),
        IpAddrKind::V6 => (OPENDNS_IPS_V6, LookupIpStrategy::Ipv6Only),
    };
    for ip in ips {
        let udp = NameServerConfig {
            socket_addr: SocketAddr::new(*ip, 53),
            protocol: Protocol::Udp,
            tls_dns_name: None,
            trust_negative_responses: true,
            bind_addr: None,
        };
        name_servers.push(udp);
    }
    let config = ResolverConfig::from_parts(None, vec![], name_servers);
    let mut options = ResolverOpts::default();
    options.ip_strategy = ip_strategy;

    let resolver = Resolver::new(config, options)?;
    let response = resolver.lookup_ip("myip.opendns.com")?;
    Ok(response.iter().next().unwrap())
}
