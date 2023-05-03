use std::{pin::Pin, rc::Rc, sync::Arc};

use futures::{
    channel::oneshot::{channel, Sender},
    Future,
};
use js_sys::{Function, Uint8Array};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Error as JsonError};
use ui::cache::Cache;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{Blob, FormData, XmlHttpRequest};

use crate::config::API_URL;

pub struct RemoteCache;

#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    id: String,
}

#[derive(Debug)]
pub enum RemoteCacheError {
    RequestCreationError(String),
    ResponseFormatError(String),
    ResponseMismatch(u16, String),
    Canceled,
    Json(JsonError)
}

impl RemoteCache {
    pub fn new() -> RemoteCache {
        RemoteCache
    }
}

#[derive(Debug)]
enum Message {
    Response(u16, String),
    ResponseError(String),
}

impl Cache for RemoteCache {
    type Error = RemoteCacheError;

    fn cache_file<'a>(
        self: Arc<Self>,
        buf: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<String, Self::Error>> + Send + 'a>> {
        let (sender, receiver) = channel::<Message>();
        let res = |sender: Sender<Message>| {
            let form_data = FormData::new()?;
            let js_arr = Uint8Array::new_with_length(buf.len() as u32);
            js_arr.copy_from(&buf);
            let blob = Blob::new_with_u8_array_sequence(&js_arr)?;
            form_data.append_with_blob("file", &blob)?;
            let req = Rc::new(XmlHttpRequest::new()?);
            req.open("POST", &format!("{}/cache", API_URL))?;
            req.send_with_opt_form_data(Some(&form_data))?;
            let req_cloned = req.clone();
            let ol_closure = Closure::once_into_js(move || match req_cloned.status() {
                Ok(status) => match req_cloned.response_text() {
                    Ok(opt_txt) => match opt_txt {
                        Some(txt) => {
                            sender.send(Message::Response(status, txt)).unwrap();
                        }
                        None => {
                            sender
                                .send(Message::Response(status, String::new()))
                                .unwrap();
                        }
                    },
                    Err(js_val) => {
                        sender
                            .send(Message::ResponseError(format!("{:?}", js_val)))
                            .unwrap();
                    }
                },
                Err(js_val) => {
                    sender
                        .send(Message::ResponseError(format!("{:?}", js_val)))
                        .unwrap();
                }
            });
            let ol_fn: Function = ol_closure.dyn_into()?;

            req.set_onload(Some(&ol_fn));
            Ok::<(), JsValue>(())
        };

        let creation_error = if let Err(err) = res(sender) {
            err.as_string()
        } else {
            None
        };

        Box::pin(async move {
            if let Some(creation_error) = creation_error {
                return Err(RemoteCacheError::RequestCreationError(creation_error));
            }
            let received = receiver.await;
            match received {
                Ok(message) => match message {
                    Message::ResponseError(res_err) => {
                        Err(RemoteCacheError::ResponseFormatError(res_err))
                    }
                    Message::Response(status, txt) if status >= 200 && status < 300 => {
                        match from_str::<'_, SuccessResponse>(&txt) {
                            Ok(res_obj) => Ok(res_obj.id),
                            Err(json_err) => Err(RemoteCacheError::Json(json_err))
                        }
                    }
                    Message::Response(status, txt) => {
                        Err(RemoteCacheError::ResponseMismatch(status, txt))
                    }
                },
                Err(_) => Err(RemoteCacheError::Canceled),
            }
        })
    }
}
