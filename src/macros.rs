#[macro_export]
macro_rules! unwrap_or_empty {
    ($option:expr) => {
        $option.as_deref().unwrap_or("")
    };
}

/// Macro to unwrap an `Option<T>` or return an error message.
#[macro_export]
macro_rules! unwrap_or_err {
    ($option:expr, $err:expr) => {
        $option.ok_or_else(|| ConversionError::MissingField($err.into()))?
    };
}

#[macro_export]
macro_rules! unwrap_or_message {
    ($option:expr, $err:expr) => {
        $option.ok_or_else(|| ConversionError::ErrorMessage($err.into()))?
    };
}

#[macro_export]
macro_rules! ok_or_err {
    ($result:expr, $err:expr) => {
        $result.or_else(|_| Err(ConversionError::MissingField($err.into())))?
    };
}

#[macro_export]
macro_rules! ok_or_message {
    ($result:expr, $err:expr) => {
        $result.or_else(|_| Err(ConversionError::ErrorMessage($err.into())))?
    };
}

#[macro_export]
macro_rules! ok_or_default {
    ($result:expr) => {
        $result.unwrap_or_default()
    };
}
