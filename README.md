# Jsosso Json Parser

Jsosso is a simple Json parser.
It provides a Json datatype and

- a serializer to transform Json data into byte vectors,
- a parser to read Json values from streams,
- a macro providing a domain-specific language to easily build JSON structures in Rust,
- a random Json value generator.

The main purpose of Jsosso is to serve as a demonstrator for [pacosso].
As such, it highlights simplicity, not performance
or other features you may expect from a full-fledged Json parser.

[pacosso]:  https://github.com/toschoo/pacosso

The `src/bin` directory contains:

- bench:
  a benchmark program.

- stream:
  a very simple TCP/IP server that parses messages sent through a socket.

- stream2:
  a simple TCP/IP server that parses messages sent through a socket
  and sends an ack after each message.

- client:
  a client program for `stream` and `stream2`.
  For stream run the client without arguments.
  For stream2 run the client with argument `-a`.

