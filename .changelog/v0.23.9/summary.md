*Aug 5, 2022*

This minor release adds Basic authentication support for HTTP and WebSocket RPC
clients, in addition to some dependency updates.

We had to restrict our `time` dependency for some crates to a version range of
`>=0.3, <0.3.12` due to what seems to be a recent issue in `js-sys` causing our
no\_std support to break. We will undo this restriction as soon as the issue is
resolved.
