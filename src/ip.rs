use core::panic;
use dns::record::{Record, RecordType, A};
use dns::{Answer, Flags, Labels, QClass, Query, Request};
use dns_transport::{Error as TransportError, Transport, UdpTransport};
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_request() -> Request {
    let query = Query {
        qname: Labels::encode("myip.opendns.com").unwrap(),
        qclass: QClass::IN,
        qtype: RecordType::A,
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

fn pick_resolver() -> String {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    RESOLVERS[(n % RESOLVERS.len() as u128) as usize].into()
}

pub fn dig() -> Result<IpAddr, TransportError> {
    let transport = UdpTransport::new(pick_resolver());
    let request = make_request();
    let result = transport.send(&request)?;

    match result.answers.first().unwrap() {
        Answer::Standard {
            record: Record::A(A { address }),
            ..
        } => Ok(IpAddr::V4(*address)),
        _ => panic!("Unknown error"),
    }
}
