use maxminddb::MaxMindDBError;

pub fn error_type(error: &MaxMindDBError) -> &'static str {
    match error {
        MaxMindDBError::AddressNotFoundError(_) | MaxMindDBError::InvalidDatabaseError(_) => {
            "maxminddb"
        }
        MaxMindDBError::IoError(_) => "maxminddb/io",
        MaxMindDBError::MapError(_) => "maxminddb/map",
        MaxMindDBError::DecodingError(_) => "maxminddb/decoding",
    }
}

pub fn error_message(error: MaxMindDBError) -> String {
    match error {
        MaxMindDBError::AddressNotFoundError(msg) => msg,
        MaxMindDBError::InvalidDatabaseError(msg) => msg,
        MaxMindDBError::IoError(msg) => msg,
        MaxMindDBError::MapError(msg) => msg,
        MaxMindDBError::DecodingError(msg) => msg,
    }
}
