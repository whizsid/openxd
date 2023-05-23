use std::{pin::Pin, rc::Rc, sync::Arc};

use futures::{
    channel::oneshot::{channel, Sender},
    Future,
};
use js_sys::{Function, Uint8Array};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Error as JsonError};
use ui::external::External;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, JsError};
use web_sys::{Blob, FormData, XmlHttpRequest, window};

use crate::config::API_URL;

pub struct RestApi;

#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    id: String,
}

#[derive(Debug)]
pub enum RestApiError {
    RequestCreationError(String),
    ResponseFormatError(String),
    ResponseMismatch(u16, String),
    Canceled,
    Json(JsonError),
}

impl RestApi {
    pub fn new() -> RestApi {
        RestApi
    }
}

#[derive(Debug)]
enum Message {
    Response(u16, String),
    ResponseError(String),
}

impl External for RestApi {
    type Error = RestApiError;

    fn create_project_using_existing_file<'async_trait>(
        self: Arc<Self>,
        buf: Vec<u8>,
        project_name: String,
    ) -> Pin<Box<dyn Future<Output = Result<String, Self::Error>> + Send + 'async_trait>>
    where
        Self: 'async_trait,
    {
        let (sender, receiver) = channel::<Message>();
        let res = |sender: Sender<Message>| {

            let win = window().unwrap();
            let local_storage = win.local_storage().unwrap().unwrap();
            let token = local_storage.get_item("_token").unwrap();
            
            match token {
                Some(token) => {
                    let form_data = FormData::new()?;
                    let js_arr = Uint8Array::new_with_length(buf.len() as u32);
                    js_arr.copy_from(&buf);
                    let blob = Blob::new_with_u8_array_sequence(&js_arr)?;
                    form_data.append_with_str("project_name", &project_name)?;
                    form_data.append_with_blob("file", &blob)?;
                    let req = Rc::new(XmlHttpRequest::new()?);
                    req.open("POST", &format!("{}/cache", API_URL))?;
                    req.set_request_header("Authorization", &format!("Bearer {}", token))?;
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
                },
                None=> {
                    Err(JsError::new("Unauthenticated").into())
                }
            }

        };

        let creation_error = if let Err(err) = res(sender) {
            err.as_string()
        } else {
            None
        };

        Box::pin(async move {
            if let Some(creation_error) = creation_error {
                return Err(RestApiError::RequestCreationError(creation_error));
            }
            let received = receiver.await;
            match received {
                Ok(message) => match message {
                    Message::ResponseError(res_err) => {
                        Err(RestApiError::ResponseFormatError(res_err))
                    }
                    Message::Response(status, txt) if status >= 200 && status < 300 => {
                        match from_str::<'_, SuccessResponse>(&txt) {
                            Ok(res_obj) => Ok(res_obj.id),
                            Err(json_err) => Err(RestApiError::Json(json_err)),
                        }
                    }
                    Message::Response(status, txt) => {
                        Err(RestApiError::ResponseMismatch(status, txt))
                    }
                },
                Err(_) => Err(RestApiError::Canceled),
            }
        })
    }
}
