use anyhow::{bail, Context };
use js_sys::{Uint8Array};
use log::{info, Level};
use serde::{Serialize};
use serde_wasm_bindgen::{ Serializer };
use thiserror::Error;
use wasm_bindgen::prelude::*;
use pixelpack::plater::request::{Algorithm, BedExpansionMode, ConfigOrder, PointEnumerationMode, Strategy, ThreadingMode};
use crate::PixelpackResult::{PackingError, Answer};

use crate::request::{handle_request, PlacingResult, WasmArgs};

mod request;

#[derive(Serialize)]
#[serde(tag = "tag", rename_all = "camelCase")]
enum PixelpackResult {
    PackingError { value: String, reportable: bool},
    Answer {value: PlacingResult}
}

#[derive(Error, Debug)]
pub enum TaggedError {
    #[error(transparent)]
    Reportable(anyhow::Error),

    #[error(transparent)]
    Hidden(anyhow::Error)
}

impl From<Result<PlacingResult, TaggedError>> for PixelpackResult {
    fn from(value: Result<PlacingResult, TaggedError>) -> Self {
        match value {
            Ok(value) => Answer {value},
            Err(value) => match value {
                TaggedError::Reportable(err) => {
                    PackingError {value: format!("{:?}", err), reportable: true}
                }
                TaggedError::Hidden(err) => {
                    PackingError {value: format!("{:?}", err), reportable: false}
                }
            }}
        }
    }

const MAP_SERIALIZER: Serializer = Serializer::new().serialize_maps_as_objects(true);

#[wasm_bindgen]
pub fn decode_pixel_data(buf: &Uint8Array, options: JsValue) -> JsValue {
    let alg = Algorithm {
        threading_mode: ThreadingMode::SingleThreaded,
        strategy: Strategy::PixelPack,
        order_config: ConfigOrder::PointFirst,
        point_enumeration_mode: PointEnumerationMode::Row,
        bed_expansion_mode: BedExpansionMode::Exponential
    };

    let result: PixelpackResult = decode_pixel_data_generic(buf, options, alg).into();
    result.serialize(&MAP_SERIALIZER).unwrap()
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

    let result: PixelpackResult = decode_pixel_data_generic(buf, options, alg).into();
    result.serialize(&MAP_SERIALIZER).unwrap()
}

pub fn decode_pixel_data_generic(buf: &Uint8Array, options: JsValue, alg: Algorithm) -> Result<PlacingResult, TaggedError> {
    match console_log::init_with_level(Level::Debug) {
        Ok(_) => (),
        Err(e) => info!("Err occurred: {}",e)
    }

    let args: WasmArgs =
        serde_wasm_bindgen::from_value(options)
        .or_else(|x| bail!("{}", x.to_string()))
            .with_context(|| format!("Could not parse WasmArgs"))
            .map_err(TaggedError::Hidden)?;

    let data: Vec<u8> = buf.to_vec();

    info!("{:#?}", args);
    let pixel_bufs = decode_pixel_maps(data.as_slice(), args.offsets.as_slice())
        .with_context(|| format!("Could not decode pixel data from supplied offset list"))
        .map_err(TaggedError::Hidden)?;

    let WasmArgs {
        model_options,
        options,
        ..
    } = args;

     handle_request(options, model_options, pixel_bufs, alg)

}

fn decode_pixel_maps<'a, 'b>(buf: &'a [u8], offsets: &'b [u32]) ->  anyhow::Result<Vec<&'a [u8]>> {
    let buf_len = buf.len() as u32;

    let offset_len = offsets.len();
    for i in 0..offset_len {
        if i < offset_len - 1 {
            if offsets[i] >= offsets[i + 1] {
                bail!("Offsets must increase monotonically, yet offset[{}] ({}) >= offset[{}] ({})", i, offsets[i], i+1, offsets[i+1]);
            }
        }

        if offsets[i] > buf_len {
            bail!("Offset[{}] {} exceeds the total buffer length of {} bytes", i, offsets[i], buf_len)
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

    Ok(result)
}
