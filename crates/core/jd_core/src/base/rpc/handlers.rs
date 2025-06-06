use crate::ctx::Ctx;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use rpc_router::resources_builder;
use serde_json::{json, Value};
use std::sync::Arc;

/// RPC ID and Method Capture
/// Note: This will be injected into the Axum Response extensions so that
///       it can be used downstream by the `mw_res_map` for logging and eventual
///       error client JSON-RPC serialization
#[derive(Debug)]
pub struct RpcInfo {
  pub id: Option<Value>,
  pub method: String,
}

pub async fn rpc_axum_handler(
  State(rpc_router): State<rpc_router::Router>,
  Extension(ctx): Extension<Ctx>,
  Json(rpc_req): Json<Value>,
) -> Response {
  // -- Parse and RpcRequest validate the rpc_request
  let rpc_req = match rpc_router::Request::try_from(rpc_req) {
    Ok(rpc_req) => rpc_req,
    Err(rpc_req_error) => {
      let error = crate::Error::RpcError(format!("RPC request parsing error: {}", rpc_req_error));
      let response = Json(json!({
        "jsonrpc": "2.0",
        "error": {
          "code": -32700,
          "message": error.to_string()
        },
        "id": null
      }));
      return response.into_response();
    }
  };

  // -- Create the RPC Info
  //    (will be set to the response.extensions)
  let rpc_info = RpcInfo { id: Some(rpc_req.id.clone()), method: rpc_req.method.clone() };

  // -- Add the request specific resources
  // Note: Since Ctx is per axum request, we construct additional RPC resources.
  //       These additional resources will be "overlayed" on top of the base router services,
  //       meaning they will take precedence over the base router ones, but won't replace them.
  let additional_resources = resources_builder![ctx].build();

  // -- Exec Rpc Route
  let rpc_call_result = rpc_router
    .call_with_resources(rpc_req, additional_resources)
    .await;

  // -- Build Json Rpc Success Response
  // Note: Error Json response will be generated in the mw_res_map as wil other error.
  let res = rpc_call_result.map(|rpc_call_response| {
    let body_response = json!({
      "jsonrpc": "2.0",
      "id": rpc_call_response.id,
      "result": rpc_call_response.value
    });
    Json(body_response)
  });

  // -- Create and Update Axum Response
  // Note: We store data in the Axum Response extensions so that
  //       we can unpack it in the `mw_res_map` for client-side rendering.
  //       This approach centralizes error handling for the client at the `mw_res_map` module
  let mut res = match res {
    Ok(json_response) => json_response.into_response(),
    Err(e) => {
      let error_response = Json(json!({
        "jsonrpc": "2.0",
        "error": {
          "code": -32603,
          "message": format!("Internal error: {}", e)
        },
        "id": rpc_info.id.clone()
      }));
      error_response.into_response()
    }
  };
  // Note: Here, add the capture RpcInfo (RPC ID and method) into the Axum response to be used
  //       later in the `mw_res_map` for RequestLineLogging, and eventual JSON-RPC error serialization.
  res.extensions_mut().insert(Arc::new(rpc_info));

  res
}
