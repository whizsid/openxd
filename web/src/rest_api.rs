use std::sync::Arc;

use async_trait::async_trait;
use js_sys::Uint8Array;
use serde::Deserialize;
use serde_json::{from_str, Error as JsonError};
use ui::external::External;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Blob, FormData, Headers, Request, RequestInit, Response};

use crate::config::API_URL;

pub struct RestApi;

#[derive(Debug)]
pub enum RestApiError {
    JsError(JsValue),
    ResponseMismatch(u16, String),
    Json(JsonError),
    TokenNotSet,
}

impl From<JsValue> for RestApiError {
    fn from(value: JsValue) -> Self {
        RestApiError::JsError(value)
    }
}

impl From<JsonError> for RestApiError {
    fn from(value: JsonError) -> Self {
        RestApiError::Json(value)
    }
}

impl RestApi {
    pub fn new() -> RestApi {
        RestApi
    }
}

#[async_trait(?Send)]
impl External for RestApi {
    type Error = RestApiError;

    async fn create_project_using_existing_file(
        self: Arc<Self>,
        buf: Vec<u8>,
        project_name: String,
    ) -> Result<String, RestApiError> {
        #[derive(Deserialize)]
        struct SuccessResponse {
            id: String,
        }

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

                let mut init = RequestInit::new();
                init.body(Some(&form_data));
                init.method("POST");

                let headers = Headers::new()?;
                headers.append("Authorization", &format!("Bearer {}", token))?;
                init.headers(&headers);

                let request = Request::new_with_str_and_init(
                    &format!("{}/api/create-project", API_URL),
                    &init,
                )?;

                let resp_value = JsFuture::from(win.fetch_with_request(&request)).await?;
                // `resp_value` is a `Response` object.
                assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into().unwrap();

                let status = resp.status();
                let txt = JsFuture::from(resp.text()?).await?;
                let res_str = txt.as_string().unwrap();

                if status >= 300 || status < 200 {
                    return Err(RestApiError::ResponseMismatch(status, res_str));
                }

                let success_res = from_str::<SuccessResponse>(&res_str)?;

                Ok(success_res.id)
            }
            None => Err(RestApiError::TokenNotSet),
        }
    }

    async fn save_current_snapshot(self: Arc<Self>) -> Result<(), Self::Error> {
        #[derive(Deserialize)]
        struct SuccessResponse {
            download_id: String,
        }

        let win = window().unwrap();
        let local_storage = win.local_storage()?;
        let token = local_storage.unwrap().get_item("_token")?;

        if let Some(token) = token {
            let mut init = RequestInit::new();
            init.method("GET");

            let url = format!("{}/api/current-tab-snapshot", API_URL);

            let request = Request::new_with_str_and_init(&url, &init)?;

            request
                .headers()
                .set("Authorization", &format!("Bearer {}", token))?;

            let resp_value = JsFuture::from(win.fetch_with_request(&request)).await?;

            // `resp_value` is a `Response` object.
            assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into().unwrap();

            let status = resp.status();
            // Convert this other `Promise` into a rust `Future`.
            let body_value = JsFuture::from(resp.text()?).await?;
            let body_str = body_value.as_string().unwrap();

            if status >= 300 || status < 200 {
                return Err(RestApiError::ResponseMismatch(status, body_str));
            }

            let success_res = from_str::<SuccessResponse>(&body_str)?;

            let download_id = success_res.download_id;

            win.open_with_url_and_target(
                &format!("{}/api/snapshot/{}", API_URL, download_id),
                "_blank",
            )?;

            Ok(())
        } else {
            Err(RestApiError::TokenNotSet)
        }
    }
}
