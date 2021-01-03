use dns::{MandatedLength, WireError};
use dns_transport::Error as TransportError;

pub fn error_type(error: &TransportError) -> String {
    if let TransportError::WireError(_) = error {
        "dns/protocol".into()
    } else {
        "dns/network".into()
    }
}

pub fn error_message(error: TransportError) -> String {
    match error {
        TransportError::WireError(e) => wire_error_message(e),
        TransportError::TruncatedResponse => "Truncated response".into(),
        TransportError::NetworkError(e) => e.to_string(),
    }
}

// Copied from dog/src/output.rs
fn wire_error_message(error: WireError) -> String {
    match error {
        WireError::IO => "Malformed packet: insufficient data".into(),
        WireError::WrongRecordLength {
            stated_length,
            mandated_length: MandatedLength::Exactly(len),
        } => {
            format!(
                "Malformed packet: record length should be {}, got {}",
                len, stated_length
            )
        }
        WireError::WrongRecordLength {
            stated_length,
            mandated_length: MandatedLength::AtLeast(len),
        } => {
            format!(
                "Malformed packet: record length should be at least {}, got {}",
                len, stated_length
            )
        }
        WireError::WrongLabelLength {
            stated_length,
            length_after_labels,
        } => {
            format!(
                "Malformed packet: length {} was specified, but read {} bytes",
                stated_length, length_after_labels
            )
        }
        WireError::TooMuchRecursion(indices) => {
            format!("Malformed packet: too much recursion: {:?}", indices)
        }
        WireError::OutOfBounds(index) => {
            format!("Malformed packet: out of bounds ({})", index)
        }
        WireError::WrongVersion {
            stated_version,
            maximum_supported_version,
        } => {
            format!(
                "Malformed packet: record specifies version {}, expected up to {}",
                stated_version, maximum_supported_version
            )
        }
    }
}
