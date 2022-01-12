*Jan 11, 2022*

This release exclusively focuses on removing `native-tls`/`openssl` from the
dependency tree and replacing it with `rustls`. This was previously incorrectly
configured in our `hyper-proxy` dependency.
