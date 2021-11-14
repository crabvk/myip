use core::panic;
use dns::record::{Record, RecordType, A, AAAA};
use dns::{Answer, Flags, Labels, QClass, Query, Request};
use dns_transport::{Error as TransportError, TcpTransport, Transport, UdpTransport};
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_request(qtype: RecordType) -> Request {
    let query = Query {
        qname: Labels::encode("myip.opendns.com").unwrap(),
        qclass: QClass::IN,
        qtype,
    };

    Request {
        transaction_id: 0xABCD,
        flags: Flags::query(),
        query,
        additional: None,
    }
}

const RESOLVERS: [&str; 4] = [
    "resolver1.opendns.com",
    "resolver2.opendns.com",
    "resolver3.opendns.com",
    "resolver4.opendns.com",
];

fn pick_resolver(resolvers: &[&str]) -> String {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    resolvers[(n % resolvers.len() as u128) as usize].into()
}

pub fn dig(qtype_a: bool) -> Result<IpAddr, TransportError> {
    let result = if qtype_a {
        let transport = UdpTransport::new(pick_resolver(&RESOLVERS));
        let request = make_request(RecordType::A);
        transport.send(&request)?
    } else {
        let transport = TcpTransport::new(pick_resolver(&RESOLVERS[..2]));
        let request = make_request(RecordType::AAAA);
        transport.send(&request)?
    };

    match result.answers.first().unwrap() {
        Answer::Standard {
            record: Record::A(A { address }),
            ..
        } => Ok(IpAddr::V4(*address)),
        Answer::Standard {
            record: Record::AAAA(AAAA { address }),
            ..
        } => Ok(IpAddr::V6(*address)),
        _ => panic!("Unknown error"),
    }
}
