# oneshot-oauth2-callback

Easily receive an OAuth2 code grant callback at a local address.

Useful for command-line tools needing to perform an OAuth2 flow, or for development/testing of more complex flows.

## Use

```rust
// Bind the listener to local port
let address = std::net::SocketAddr::from(([127, 0, 0, 1], 5000));
// Listen for the first OAuth2 response, then immediately shutdown and return
let code_grant = oneshot_oauth2_callback::oneshot(&address)
  .await
  .expect("oauth2 login failed");
// use code_grant.code, code_grant.state.secret() to continue the flow
```
