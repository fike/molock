# Molock Roadmap

This document outlines the planned features and enhancements for Molock, focusing on closing the gap with other industry-standard mock servers while maintaining our core goals of extreme performance and native observability.

## Planned Features

### Advanced Request Matching & Validation
- [ ] **Regex Matching**: Allow regular expressions in path definitions, headers, and query parameters for more flexible routing.
- [x] **JSON Schema Validation**: Support validating incoming request bodies against predefined JSON schemas before matching a rule.

### Proxying and Forwarding
- [ ] **Proxy/Fallback Mode**: Introduce the ability to transparently forward requests to a real backend service if no mock rules match (e.g., `proxy-mode: missing` or `all`).
- [ ] **Record and Playback**: Allow Molock to act as a proxy and automatically generate mock configuration files based on the intercepted real traffic.

### Response Enhancements
- [ ] **External Body Files**: Support loading response bodies from external files (e.g., `bodyFile: "response.json"`) to keep configuration files clean when dealing with large payloads.

### Network-Level Chaos Engineering
- [ ] **TCP Fault Injection**: Add support for low-level connection disruptions, such as abruptly closing the TCP connection, sending truncated data, or sending responses in slow chunks.
