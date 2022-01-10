OBEX Rust Server
================

Read-only access to the Parallax Object Exchange (OBEX)

This provides the server-side component of a homegrown solution to Parallax
decommissioning their long-lived and much-loved OBEX. Though originally
written with [Python 3][1], I was not pleased with the performance and needed
an excuse to learn Rust.

This project was used to experiment with Rust's ability to implement common
design principles that I have become accustomed to in enterprise Java, Python,
and C++ development. It accomplished that goal quite nicely, and my findings
have been summarized by [this blog post][2].

[1]: https://github.com/DavidZemon/obex-server
[2]: https://objectpartners.com/2021/07/29/an-exploration-in-rust-musings-from-a-java-c-developer/
