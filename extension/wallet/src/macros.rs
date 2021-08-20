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

// https://veykril.github.io/tlborm/decl-macros/patterns/repetition-replacement.html
macro_rules! replace_expr {
    ($_t:tt $sub:ident) => {
        $sub
    };
}

#[macro_export]
macro_rules! impl_window {
    ($window: ident, async fn $fn_name:ident( $( $arg_name: ident: $arg_type: ty ),* ) -> $return_type: ty $body: block) => {{
        let handler = Closure::wrap(Box::new(|$($arg_name: JsValue),*| {
            let future = async move {
                $( let $arg_name = $arg_name.into_serde::<$arg_type>().map_err(|_| anyhow::anyhow!("wrong type"))?; )*

                $body
            };

            let future = future
                .map_ok(|ok| {
                    log::debug!("Successfully invoked {}", stringify!($fn_name));

                    JsValue::from_serde(&ok).expect("serialization never fails")
                })
                .map_err(|e: anyhow::Error| JsValue::from_str(&format!("{:#}", e)));

            future_to_promise(future)
        }) as Box<dyn Fn($(replace_expr!($arg_name JsValue)),*) -> Promise>);

        js_sys::Reflect::set(
            &$window,
            &JsValue::from_str(&stringify!($fn_name)),
            &handler.into_js_value().into(),
        )
        .expect("to set property on window");
    }};
}

#[macro_export]
macro_rules! add_message_handler {
    ($browser: ident, async fn $fn_name:ident( $( $arg_name: ident: $arg_type: ty ),* $(,)* ) -> $return_type: ty $body: block) => {{
        #[derive(Deserialize, Debug)]
        struct Args {
            $( $arg_name: $arg_type ),*
        }

        #[derive(Deserialize, Debug)]
        struct Msg {
            method: String,
            args: Args,
        }

        let handler = Closure::wrap(Box::new(|msg: JsValue| {
            let msg = match msg.into_serde::<Msg>() {
                Ok(msg) => msg,
                Err(_) => return JsValue::from_bool(false),
            };

            if msg.method != stringify!($fn_name) {
                return JsValue::from_bool(false);
            }

            let Args {
                $( $arg_name ),*
            } = msg.args;

            let future = async move { $body };

            use futures::FutureExt;
            let future = future.map(|result: anyhow::Result<_>| {
                Ok(
                    JsValue::from_serde(&result.map_err(|e: anyhow::Error| format!("{:#}", e)))
                        .expect_throw("serialization always works"),
                )
            });

            future_to_promise(future).into()
        }) as Box<dyn Fn(JsValue)-> JsValue>);

        $browser
            .runtime()
            .on_message()
            .add_listener(handler.into_js_value().unchecked_ref());
    }};
}
