use amdsmi::AmdsmiStatusT;
use log::error;
use std::{error::Error, fmt::Display};

/// Error treatment concerning AMD SMI library.
#[derive(Debug)]
pub struct AmdError(pub AmdsmiStatusT);

impl Display for AmdError {
    fn fmt(&self, format: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(format, "amdsmi error {}", self.0)
    }
}

impl Error for AmdError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub struct Features<T> {
    pub value: Option<T>,
    pub supported: bool,
}

impl<T> Features<T> {
    pub fn new(value: Option<T>, supported: bool) -> Self {
        Self { value, supported }
    }
}

pub fn is_valid<T>(func: impl FnOnce() -> Result<T, AmdError>) -> Features<T> {
    match func() {
        Ok(value) => Features::new(Some(value), true),
        Err(AmdError(status)) => {
            if status == AmdsmiStatusT::AmdsmiStatusNotSupported {
                error!("Function or feature not support by AMDSMI on this machine");
                Features::new(None, false)
            } else {
                error!("Failed to get metric : {status:?}");
                Features::new(None, false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test `fmt` function in `Display` implementation for `AmdError` with AMD SMI error display
    #[test]
    fn test_fmt_display() {
        let error = AmdError(AmdsmiStatusT::AmdsmiStatusSuccess);
        let msg = format!("amdsmi error {}", error.0);
        assert_eq!(format!("{}", error), msg);
    }

    // Test `source` function in `Error` implementation for `AmdError`
    #[test]
    fn test_source() {
        let error = AmdError(AmdsmiStatusT::AmdsmiStatusSuccess);
        assert!(error.source().is_none());
    }
}
