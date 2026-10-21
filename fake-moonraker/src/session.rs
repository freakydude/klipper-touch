//! JSON-RPC-Sitzung einer WebSocket-Verbindung.

use crate::backend::Backend;
use crate::files::GcodeFile;
use crate::gcode::{self, CommandError};
use crate::printer::Printer;
use crate::subscription::Subscription;
use serde_json::{json, Value};

const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const PARSE_ERROR: i64 = -32700;
const REQUEST_FAILED: i64 = 400;
const NOT_FOUND: i64 = 404;

#[derive(Debug)]
struct RpcError {
    code: i64,
    message: String,
}

impl RpcError {
    fn new(code: i64, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }

    fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(REQUEST_FAILED, message)
    }

    fn to_json(&self) -> Value {
        json!({ "code": self.code, "message": self.message })
    }
}

impl From<CommandError> for RpcError {
    fn from(error: CommandError) -> Self {
        Self::new(REQUEST_FAILED, error.message())
    }
}

pub struct Session {
    connection_id: u64,
    subscription: Subscription,
}

impl Session {
    pub fn new(connection_id: u64) -> Self {
        Self { connection_id, subscription: Subscription::default() }
    }

    /// Verarbeitet eine eingehende Nachricht; `None`, wenn keine Antwort faellig ist.
    pub fn handle_text(&mut self, backend: &Backend, text: &str) -> Option<String> {
        match serde_json::from_str::<Value>(text) {
            Ok(Value::Array(requests)) => {
                let responses: Vec<Value> = requests
                    .iter()
                    .filter_map(|request| self.handle_request(backend, request))
                    .collect();
                (!responses.is_empty()).then(|| Value::Array(responses).to_string())
            }
            Ok(request) => self.handle_request(backend, &request).map(|response| response.to_string()),
            Err(_) => Some(failure(Value::Null, &RpcError::new(PARSE_ERROR, "Parse error")).to_string()),
        }
    }

    /// Aenderungen der abonnierten Objekte seit der letzten Meldung als `notify_status_update`.
    pub fn status_notification(&mut self, backend: &Backend) -> Option<String> {
        let changes = self.subscription.changes(&backend.status())?;
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notify_status_update",
            "params": [changes, backend.eventtime()],
        });
        Some(notification.to_string())
    }

    fn handle_request(&mut self, backend: &Backend, request: &Value) -> Option<Value> {
        let id = request.get("id").cloned();
        let outcome = match request.get("method").and_then(Value::as_str) {
            Some(method) => {
                let params = request.get("params").cloned().unwrap_or(Value::Null);
                self.dispatch(backend, method, &params)
            }
            None => Err(RpcError::new(INVALID_REQUEST, "Invalid Request")),
        };
        let id = id?;
        Some(match outcome {
            Ok(result) => json!({ "jsonrpc": "2.0", "result": result, "id": id }),
            Err(error) => failure(id, &error),
        })
    }

    fn dispatch(&mut self, backend: &Backend, method: &str, params: &Value) -> Result<Value, RpcError> {
        match method {
            "server.connection.identify" => Ok(json!({ "connection_id": self.connection_id })),
            "printer.objects.subscribe" => self.subscribe(backend, params),
            "printer.objects.list" => Ok(json!({ "objects": backend.object_names() })),
            "printer.objects.query" => query_objects(backend, params),
            "printer.gcode.script" => run_script(backend, params),
            "printer.print.start" => start_print(backend, params),
            "printer.print.pause" => control_printer(backend, Printer::pause_print),
            "printer.print.resume" => control_printer(backend, Printer::resume_print),
            "printer.print.cancel" => control_printer(backend, Printer::cancel_print),
            "printer.emergency_stop" => Ok(acknowledge(backend, Printer::emergency_stop)),
            "printer.restart" | "printer.firmware_restart" => Ok(acknowledge(backend, Printer::restart)),
            "server.files.list" => Ok(backend.catalog().list()),
            "server.files.metadata" => Ok(find_file(backend, params)?.metadata()),
            "server.files.thumbnails" => Ok(find_file(backend, params)?.thumbnails()),
            _ => Err(RpcError::new(METHOD_NOT_FOUND, format!("Method not found: {method}"))),
        }
    }

    fn subscribe(&mut self, backend: &Backend, params: &Value) -> Result<Value, RpcError> {
        let objects = params
            .get("objects")
            .ok_or_else(|| RpcError::invalid_params("missing parameter 'objects'"))?;
        self.subscription.replace_with(objects).map_err(RpcError::invalid_params)?;
        let status = self.subscription.snapshot(&backend.status());
        Ok(json!({ "eventtime": backend.eventtime(), "status": status }))
    }
}

fn query_objects(backend: &Backend, params: &Value) -> Result<Value, RpcError> {
    let objects = params
        .get("objects")
        .ok_or_else(|| RpcError::invalid_params("missing parameter 'objects'"))?;
    let mut query = Subscription::default();
    query.replace_with(objects).map_err(RpcError::invalid_params)?;
    Ok(json!({ "eventtime": backend.eventtime(), "status": query.snapshot(&backend.status()) }))
}

fn run_script(backend: &Backend, params: &Value) -> Result<Value, RpcError> {
    let script = required_str(params, "script")?;
    let commands = gcode::parse_script(script)?;
    backend.with_printer(|printer| commands.into_iter().try_for_each(|command| printer.apply(command)))?;
    Ok(ok())
}

fn start_print(backend: &Backend, params: &Value) -> Result<Value, RpcError> {
    let file = find_file(backend, params)?;
    backend.with_printer(|printer| printer.start_print(file))?;
    Ok(ok())
}

fn control_printer(
    backend: &Backend,
    action: impl FnOnce(&mut Printer) -> Result<(), CommandError>,
) -> Result<Value, RpcError> {
    backend.with_printer(action)?;
    Ok(ok())
}

fn acknowledge(backend: &Backend, action: impl FnOnce(&mut Printer)) -> Value {
    backend.with_printer(action);
    ok()
}

fn find_file<'a>(backend: &'a Backend, params: &Value) -> Result<&'a GcodeFile, RpcError> {
    let filename = required_str(params, "filename")?;
    backend
        .catalog()
        .find(filename)
        .ok_or_else(|| RpcError::new(NOT_FOUND, format!("File does not exist: {filename}")))
}

fn required_str<'a>(params: &'a Value, name: &str) -> Result<&'a str, RpcError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::invalid_params(format!("missing parameter '{name}'")))
}

fn ok() -> Value {
    Value::String("ok".to_owned())
}

fn failure(id: Value, error: &RpcError) -> Value {
    json!({ "jsonrpc": "2.0", "error": error.to_json(), "id": id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::MachineProfile;

    fn exchange(session: &mut Session, backend: &Backend, request: Value) -> Value {
        let reply = session.handle_text(backend, &request.to_string()).expect("reply expected");
        serde_json::from_str(&reply).unwrap()
    }

    fn call(method: &str, params: Value) -> Value {
        json!({ "jsonrpc": "2.0", "id": "7", "method": method, "params": params })
    }

    fn setup() -> (Session, Backend) {
        (Session::new(1), Backend::new(1.0))
    }

    #[test]
    fn identify_returns_connection_id_and_echoes_request_id() {
        let (mut session, backend) = setup();

        let reply = exchange(&mut session, &backend, call("server.connection.identify", json!({})));

        assert_eq!(reply["id"], "7");
        assert_eq!(reply["result"]["connection_id"], 1);
    }

    #[test]
    fn subscribe_returns_initial_status() {
        let (mut session, backend) = setup();

        let reply = exchange(
            &mut session,
            &backend,
            call("printer.objects.subscribe", json!({ "objects": { "webhooks": ["state"] } })),
        );

        assert_eq!(reply["result"]["status"], json!({ "webhooks": { "state": "ready" } }));
    }

    #[test]
    fn status_notification_reports_changes_after_gcode() {
        let (mut session, backend) = setup();
        exchange(&mut session, &backend, call("printer.objects.subscribe", json!({ "objects": { "extruder": ["target"] } })));
        assert!(session.status_notification(&backend).is_none());

        let reply = exchange(&mut session, &backend, call("printer.gcode.script", json!({ "script": "M104 S100" })));
        let notification: Value = serde_json::from_str(&session.status_notification(&backend).unwrap()).unwrap();

        assert_eq!(reply["result"], "ok");
        assert_eq!(notification["method"], "notify_status_update");
        assert_eq!(notification["params"][0], json!({ "extruder": { "target": 100.0 } }));
    }

    #[test]
    fn gcode_errors_are_reported_as_rpc_errors() {
        let (mut session, backend) = setup();

        let reply = exchange(&mut session, &backend, call("printer.gcode.script", json!({ "script": "G0 X10" })));

        assert_eq!(reply["error"]["code"], 400);
        assert_eq!(reply["error"]["message"], "Must home axis first");
    }

    #[test]
    fn unknown_method_is_reported() {
        let (mut session, backend) = setup();

        let reply = exchange(&mut session, &backend, call("printer.unknown", json!({})));

        assert_eq!(reply["error"]["code"], -32601);
    }

    #[test]
    fn starting_a_missing_file_fails_with_not_found() {
        let (mut session, backend) = setup();

        let reply = exchange(&mut session, &backend, call("printer.print.start", json!({ "filename": "nope.gcode" })));

        assert_eq!(reply["error"]["code"], 404);
    }

    #[test]
    fn notifications_without_id_get_no_reply() {
        let (mut session, backend) = setup();
        let notification = json!({ "jsonrpc": "2.0", "method": "printer.gcode.script", "params": { "script": "M104 S50" } });

        assert!(session.handle_text(&backend, &notification.to_string()).is_none());
        assert_eq!(backend.status()["extruder"]["target"], 50.0);
    }

    #[test]
    fn batch_requests_get_batch_responses() {
        let (mut session, backend) = setup();
        let batch = json!([
            call("server.connection.identify", json!({})),
            call("server.files.list", json!({})),
        ]);

        let reply: Value = serde_json::from_str(&session.handle_text(&backend, &batch.to_string()).unwrap()).unwrap();

        assert_eq!(reply.as_array().unwrap().len(), 2);
    }

    #[test]
    fn invalid_json_yields_parse_error() {
        let (mut session, backend) = setup();

        let reply: Value = serde_json::from_str(&session.handle_text(&backend, "{").unwrap()).unwrap();

        assert_eq!(reply["error"]["code"], -32700);
        assert_eq!(reply["id"], Value::Null);
    }

    #[test]
    fn objects_list_names_the_configured_modules() {
        let mut session = Session::new(1);
        let backend = Backend::with_profile(1.0, MachineProfile::happy_hare(4));

        let reply = exchange(&mut session, &backend, call("printer.objects.list", json!({})));

        let names: Vec<&str> = reply["result"]["objects"].as_array().unwrap().iter().filter_map(Value::as_str).collect();
        assert!(names.contains(&"mmu"));
        assert!(names.contains(&"quad_gantry_level"));
    }

    #[test]
    fn objects_query_returns_selected_fields_without_subscribing() {
        let (mut session, backend) = setup();

        let reply = exchange(
            &mut session,
            &backend,
            call("printer.objects.query", json!({ "objects": { "configfile": ["save_config_pending"] } })),
        );

        assert_eq!(reply["result"]["status"], json!({ "configfile": { "save_config_pending": false } }));
        assert!(session.status_notification(&backend).is_none());
    }
}
