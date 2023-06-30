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

impl From<RestApiError> for String {
    fn from(value: RestApiError) -> Self {
        format!("{:?}", value)
    }
}

impl RestApi {
    pub fn new() -> RestApi {
        RestApi
    }
}

#[async_trait(?Send)]
impl External for RestApi {
    async fn create_project_using_existing_file(
        &self,
        buf: Vec<u8>,
        project_name: String,
    ) -> Result<String, String> {
        #[derive(Deserialize)]
        struct SuccessResponse {
            id: String,
        }

        let win = window().unwrap();
        let local_storage = win.local_storage().unwrap().unwrap();
        let token = local_storage.get_item("_token").unwrap();

        match token {
            Some(token) => {
                let form_data = FormData::new().map_err(RestApiError::from)?;
                let js_arr = Uint8Array::new_with_length(buf.len() as u32);
                js_arr.copy_from(&buf);
                let js_arr_wrapped = js_sys::Array::new();
                js_arr_wrapped.push(&js_arr);
                let blob = Blob::new_with_u8_array_sequence(&js_arr_wrapped)
                    .map_err(RestApiError::from)?;
                form_data
                    .append_with_str("project_name", &project_name)
                    .map_err(RestApiError::from)?;
                form_data
                    .append_with_blob("file", &blob)
                    .map_err(RestApiError::from)?;

                let mut init = RequestInit::new();
                init.body(Some(&form_data));
                init.method("POST");

                let headers = Headers::new().map_err(RestApiError::from)?;
                headers.append("Authorization", &format!("Bearer {}", token)).map_err(RestApiError::from)?;
                init.headers(&headers);

                let request = Request::new_with_str_and_init(
                    &format!("{}/api/create-project", API_URL),
                    &init,
                )
                .map_err(RestApiError::from)?;

                let resp_value = JsFuture::from(win.fetch_with_request(&request))
                    .await
                    .map_err(RestApiError::from)?;
                // `resp_value` is a `Response` object.
                assert!(resp_value.is_instance_of::<Response>());
                let resp: Response = resp_value.dyn_into().unwrap();

                let status = resp.status();
                let txt = JsFuture::from(resp.text().map_err(RestApiError::from)?)
                    .await
                    .map_err(RestApiError::from)?;
                let res_str = txt.as_string().unwrap();

                if status >= 300 || status < 200 {
                    return Err(RestApiError::ResponseMismatch(status, res_str).into());
                }

                let success_res =
                    from_str::<SuccessResponse>(&res_str).map_err(RestApiError::from)?;

                Ok(success_res.id)
            }
            None => Err(RestApiError::TokenNotSet.into()),
        }
    }

    async fn save_current_snapshot(&self) -> Result<(), String> {
        #[derive(Deserialize)]
        struct SuccessResponse {
            download_id: String,
        }

        let win = window().unwrap();
        let local_storage = win.local_storage().map_err(RestApiError::from)?;
        let token = local_storage
            .unwrap()
            .get_item("_token")
            .map_err(RestApiError::from)?;

        if let Some(token) = token {
            let mut init = RequestInit::new();
            init.method("GET");

            let url = format!("{}/api/current-tab-snapshot", API_URL);

            let request =
                Request::new_with_str_and_init(&url, &init).map_err(RestApiError::from)?;

            request
                .headers()
                .set("Authorization", &format!("Bearer {}", token))
                .map_err(RestApiError::from)?;

            let resp_value = JsFuture::from(win.fetch_with_request(&request))
                .await
                .map_err(RestApiError::from)?;

            // `resp_value` is a `Response` object.
            assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into().unwrap();

            let status = resp.status();
            // Convert this other `Promise` into a rust `Future`.
            let body_value = JsFuture::from(resp.text().map_err(RestApiError::from)?)
                .await
                .map_err(RestApiError::from)?;
            let body_str = body_value.as_string().unwrap();

            if status >= 300 || status < 200 {
                return Err(RestApiError::ResponseMismatch(status, body_str).into());
            }

            let success_res = from_str::<SuccessResponse>(&body_str).map_err(RestApiError::from)?;

            let download_id = success_res.download_id;

            win.open_with_url(&format!("{}/api/snapshot/{}", API_URL, download_id))
                .map_err(RestApiError::from)?;

            Ok(())
        } else {
            Err(RestApiError::TokenNotSet.into())
        }
    }
}
