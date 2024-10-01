[![rust](https://github.com/airenas/authware/actions/workflows/rust.yml/badge.svg)](https://github.com/airenas/authware/actions/workflows/rust.yml)

# Authware Traefik Middleware

Traefik auth middleware written in Rust.

A custom middleware for Traefik that provides authentication functionality by integrating with a backend authentication service. The middleware forwards authentication requests to an external service and handles secure routing for applications behind Traefik.


## Features

- **Forward Authentication**: Forward authentication requests to an external authentication service
- **Customizable Session storage**: uses Redis or InMemory session storage 
- **Data encryption in storage**: no session or user info exposed to external storage. It allows simple connection to redis without the need to setup TLS
- **Service runs only under TLS**: for secure traefik `<->` authware communication
- **Packed in docker**: uses the smallest possible image to run the rust app.
- **One cmd to run the sample and ready to test**: did you try to test other traefik middlewares? Authelia? Then you should know what it means to try it...
- **Designed for an easy adding of new authentication backends or storages** implement `trait SessionStore` (4 methods), `trait AuthService` (one method), and configure them in main.

## Usage

pending...


---
### License

Copyright © 2024, [Airenas Vaičiūnas](https://github.com/airenas).
Released under [The 3-Clause BSD License](LICENSE).

---
