pub mod parser;
pub mod sparql;

use crate::parser::{Parser, PartialResult};
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{ReadableStream, ReadableStreamDefaultReader};

#[derive(Debug)]
pub enum SparqlResultReaderError {
    CorruptStream,
    Canceled,
    JsonParseError(String),
}

#[cfg(feature = "call_from_rust")]
pub async fn read<F: AsyncFn(PartialResult)>(
    stream: ReadableStream,
    batch_size: usize,
    limit: Option<usize>,
    offset: usize,
    callback: F,
) -> Result<(), SparqlResultReaderError> {
    let reader: ReadableStreamDefaultReader = stream.get_reader().unchecked_into();
    let mut parser = Parser::new(batch_size, limit, offset);

    loop {
        let chunk = wasm_bindgen_futures::JsFuture::from(reader.read())
            .await
            .map_err(|err| {
                let reason = err.as_string();
                if reason.is_some_and(|reason| reason == "Query was canceled") {
                    SparqlResultReaderError::Canceled
                } else {
                    SparqlResultReaderError::CorruptStream
                }
            })?;
        if js_sys::Reflect::get(&chunk, &JsValue::from_str("done"))
            .map_err(|_| SparqlResultReaderError::CorruptStream)?
            .as_bool()
            .unwrap_or(false)
        {
            break;
        }
        let bytes = Uint8Array::new(
            &js_sys::Reflect::get(&chunk, &JsValue::from_str("value"))
                .map_err(|_| SparqlResultReaderError::CorruptStream)?,
        )
        .to_vec();
        for byte in bytes {
            if let Some(partial_result) = parser
                .read_byte(byte)
                .map_err(|err| SparqlResultReaderError::JsonParseError(err.to_string()))?
            {
                callback(partial_result).await;
            }
        }
    }
    if let Some(buffered_bindings) = parser.flush() {
        callback(buffered_bindings).await;
    }
    Ok(())
}

#[cfg(not(feature = "call_from_rust"))]
use js_sys::Function;
#[cfg(not(feature = "call_from_rust"))]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(not(feature = "call_from_rust"))]
#[wasm_bindgen]
pub async fn read(
    stream: ReadableStream,
    batch_size: usize,
    limit: Option<usize>,
    offset: usize,
    callback: &Function,
) -> Result<(), JsValue> {
    let reader: ReadableStreamDefaultReader = stream.get_reader().unchecked_into();
    let mut parser = Parser::new(batch_size, limit, offset);

    loop {
        let chunk = wasm_bindgen_futures::JsFuture::from(reader.read()).await?;
        if js_sys::Reflect::get(&chunk, &JsValue::from_str("done"))?
            .as_bool()
            .unwrap_or(false)
        {
            break;
        }
        let bytes =
            Uint8Array::new(&js_sys::Reflect::get(&chunk, &JsValue::from_str("value"))?).to_vec();
        for bytes in bytes {
            if let Some(partial_result) = parser
                .read_byte(bytes)
                .map_err(|err| JsValue::from_str(&format!("JSON parse error: {err}")))?
            {
                callback
                    .call1(
                        &JsValue::NULL,
                        &serde_wasm_bindgen::to_value(&partial_result)
                            .expect("Every ParsedChunk should be serialiable"),
                    )
                    .expect("The JS function should not throw an error");
            }
        }
    }

    if let Some(PartialResult::Bindings(bindings)) = parser.flush() {
        callback.call1(&JsValue::NULL, &serde_wasm_bindgen::to_value(&bindings)?)?;
    }
    Ok(())
}
