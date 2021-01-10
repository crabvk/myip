use maxminddb::geoip2::{Asn, City};
use maxminddb::{MaxMindDBError, Reader};
use std::path::Path;
use std::{collections::HashMap, net::IpAddr};

pub type GeoIpData = HashMap<String, String>;

pub fn lookup(path: &str, ip: IpAddr) -> Result<GeoIpData, MaxMindDBError> {
    let city_path = Path::new(path).join("GeoLite2-City.mmdb");
    let reader = Reader::open_readfile(city_path)?;
    let city = &reader.lookup::<City>(ip)?;
    let mut data = HashMap::new();

    data.insert("ip".into(), ip.to_string());

    if let Some(city) = &city.city {
        data.insert("city".into(), city.names.as_ref().unwrap()["en"].to_owned());
    }

    if let Some(country) = &city.country {
        data.insert(
            "country".into(),
            country.names.as_ref().unwrap()["en"].to_owned(),
        );
        data.insert("country_code".into(), country.iso_code.unwrap().to_owned());
    }

    if let Some(sds) = &city.subdivisions {
        let region = sds.first().unwrap();
        data.insert(
            "region".into(),
            region.names.as_ref().unwrap()["en"].to_owned(),
        );
        data.insert("region_code".into(), region.iso_code.unwrap().to_owned());
    }

    if let Some(registered) = &city.registered_country {
        data.insert(
            "registered".into(),
            registered.names.as_ref().unwrap()["en"].to_owned(),
        );
        data.insert(
            "registered_code".into(),
            registered.iso_code.unwrap().to_owned(),
        );
    }

    if let Some(location) = &city.location {
        data.insert("time_zone".into(), location.time_zone.unwrap().to_owned());
    }

    let asn_path = Path::new(path).join("GeoLite2-ASN.mmdb");
    let reader = Reader::open_readfile(asn_path)?;
    let asn = &reader.lookup::<Asn>(ip)?;

    if let Some(isp) = asn.autonomous_system_organization {
        data.insert("isp".into(), isp.into());
    }

    Ok(data)
}
