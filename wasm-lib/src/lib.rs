use std::io::Read;
use js_sys::{JsString, Uint8Array};
use log::{info, Level};
use wasm_bindgen::prelude::*;
use pixelpack::plater::request::{Algorithm, BedExpansionMode, ConfigOrder, PointEnumerationMode, Strategy, ThreadingMode};

use crate::request::{handle_request, WasmArgs};

mod request;

#[wasm_bindgen]
pub fn decode_pixel_data(buf: &Uint8Array, options: JsValue) -> JsValue {
    let alg = Algorithm {
        threading_mode: ThreadingMode::SingleThreaded,
        strategy: Strategy::PixelPack,
        order_config: ConfigOrder::PointFirst,
        point_enumeration_mode: PointEnumerationMode::Row,
        bed_expansion_mode: BedExpansionMode::Exponential
    };
    decode_pixel_data_generic(buf, options, alg)
}

#[wasm_bindgen]
pub fn decode_pixel_spiral_pack(buf: &Uint8Array, options: JsValue) -> JsValue {
    let alg = Algorithm {
        threading_mode: ThreadingMode::SingleThreaded,
        strategy: Strategy::SpiralPlace,
        order_config: ConfigOrder::PointFirst,
        point_enumeration_mode: PointEnumerationMode::Spiral,
        bed_expansion_mode: BedExpansionMode::Exponential
    };
    decode_pixel_data_generic(buf, options, alg)
}

pub fn decode_pixel_data_generic(buf: &Uint8Array, options: JsValue, alg: Algorithm) -> JsValue {
    match console_log::init_with_level(Level::Debug) {
        Ok(_) => (),
        Err(e) => info!("Err occurred: {}",e)
    }

    let args: WasmArgs = serde_wasm_bindgen::from_value(options)
        .ok()
        .expect("Couldn't parse WasmArgs");

    info!("First pass");
    let data: Vec<u8> = buf.to_vec();

    info!("{:#?}", args);
    let pixel_bufs = decode_pixel_maps(data.as_slice(), args.offsets.as_slice())
        .expect("Couldn't read pixel buf data");

    info!("Second pass");

    let WasmArgs {
        model_options,
        options,
        ..
    } = args;

    info!("third pass");

    let result = handle_request(options, model_options, pixel_bufs, alg);
    match serde_wasm_bindgen::to_value(&result) {
        Ok(val) => val,
        Err(err) => JsValue::from_str(err.to_string().as_str())
    }
}

fn decode_pixel_maps<'a, 'b>(buf: &'a [u8], offsets: &'b [u32]) -> Option<Vec<&'a [u8]>> {
    let buf_len = buf.len() as u32;

    let offset_len = offsets.len();
    for i in 0..offset_len {
        if i < offset_len - 1 {
            if offsets[i] >= offsets[i + 1] {
                return None;
            }
        }

        if offsets[i] > buf_len {
            return None;
        }
    }

    let mut result = vec![];

    for i in 0..offsets.len() {
        if i < offsets.len() - 1 {
            let start = offsets[i] as usize;
            let end = offsets[i + 1] as usize;
            result.push(&buf[start..end])
        }
    }

    Some(result)
}
