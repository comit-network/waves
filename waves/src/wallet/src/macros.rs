#[macro_export]
macro_rules! cast {
    ($e:expr) => {
        match $e.dyn_into() {
            Ok(casted) => Ok(casted),
            Err(_original) => Err(JsValue::from_str("failed to cast JsValue to given type")),
        }
    };
}

#[macro_export]
macro_rules! map_err_to_anyhow {
    ($e:expr) => {
        match $e {
            Ok(i) => Ok(i),
            Err(e) => Err(::anyhow::anyhow!(
                "{}",
                e.as_string()
                    .unwrap_or_else(|| "no error message".to_string())
            )),
        }
    };
}

#[macro_export]
macro_rules! map_err_from_anyhow {
    ($e:expr) => {
        match $e {
            Ok(i) => Ok(i),
            Err(e) => Err(JsValue::from_str(&format!("{:#}", e))),
        }
    };
}
