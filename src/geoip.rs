use maxminddb::geoip2::{Asn, City};
use maxminddb::{MaxMindDBError, Reader};
use std::net::IpAddr;
use std::path::Path;

#[derive(Debug)]
pub struct GeoIpData {
    pub ip: String,
    pub city: Option<String>,
    pub country: Option<(String, String)>,
    pub region: Option<(String, String)>,
    pub registered: Option<(String, String)>,
    pub time_zone: Option<String>,
    pub isp: Option<String>,
}

pub fn lookup(path: &str, ip: IpAddr) -> Result<GeoIpData, MaxMindDBError> {
    let city_path = Path::new(path).join("GeoLite2-City.mmdb");
    let reader = Reader::open_readfile(city_path)?;
    let city = &reader.lookup::<City>(ip)?;
    let name = city
        .city
        .as_ref()
        .and_then(|c| Some(c.names.as_ref().unwrap()["en"].to_owned()));

    let country = city.country.as_ref().and_then(|c| {
        let name = c.names.as_ref().unwrap()["en"].to_owned();
        let code = c.iso_code.unwrap().to_owned();
        Some((name, code))
    });

    let region = city.subdivisions.as_ref().and_then(|s| {
        let sd = s.first().unwrap();
        let name = sd.names.as_ref().unwrap()["en"].to_owned();
        let code = sd.iso_code.unwrap().to_owned();
        Some((name, code))
    });

    let registered = city.registered_country.as_ref().and_then(|rc| {
        let name = rc.names.as_ref().unwrap()["en"].to_owned();
        let code = rc.iso_code.unwrap().to_owned();
        Some((name, code))
    });

    let time_zone = city
        .location
        .as_ref()
        .and_then(|l| Some(l.time_zone.unwrap().to_owned()));

    let asn_path = Path::new(path).join("GeoLite2-ASN.mmdb");
    let reader = Reader::open_readfile(asn_path)?;
    let asn = &reader.lookup::<Asn>(ip)?;
    let isp = asn
        .autonomous_system_organization
        .and_then(|a| Some(a.to_owned()));

    Ok(GeoIpData {
        ip: ip.to_string(),
        city: name,
        country,
        region,
        registered,
        time_zone,
        isp,
    })
}
