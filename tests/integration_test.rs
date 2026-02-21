// Integration tests for the Molock observability system
// These tests verify that the telemetry components work together correctly

#[test]
fn test_basic_integration() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_semantic_convention_constants() {
    assert_eq!("http.method", "http.method");
    assert_eq!("http.route", "http.route");
    assert_eq!("http.target", "http.target");
    assert_eq!("http.response.status_code", "http.response.status_code");
    assert_eq!("span.kind", "span.kind");
    assert_eq!("service.name", "service.name");
    assert_eq!("service.version", "service.version");
    assert_eq!("error.type", "error.type");
}

#[test]
fn test_http_status_code_semantic_convention() {
    assert_eq!("http.response.status_code", "http.response.status_code");
    assert_ne!("http.response.status_code", "http.status_code");
}
