[![rust](https://github.com/airenas/authware/actions/workflows/rust.yml/badge.svg)](https://github.com/airenas/authware/actions/workflows/rust.yml)[![docker](https://github.com/airenas/authware/actions/workflows/docker.yml/badge.svg)](https://github.com/airenas/authware/actions/workflows/docker.yml)[![snyk vulnerabilities check](https://github.com/airenas/authware/actions/workflows/snyk.yml/badge.svg)](https://github.com/airenas/authware/actions/workflows/snyk.yml)

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
- **Designed for an easy adding of new authentication backends or storages**: implement `trait SessionStore` (4 methods), `trait AuthService` (one method), and configure them in the `main`.

## Usage

Minimal example using docker.

### Start authentication proxy example

Configure port in `.env` if needed. Default is `8000`. 

```bash
cd example
docker compose up
```
It will start traefik proxy listening for https request on the configureed port. Endpoints:
- `/public` - not protected service
- `/private` - protected by authware
- `/auth` - authware endpoint


### Test

#### public endpoint
```bash
curl https://localhost:8000/public -k -i
```
```bash
HTTP/2 200 
```
#### private endpoint - protected by authware
```bash
curl https://localhost:8000/private -k -i
```
```bash
HTTP/2 401 
No session⏎  
```

### login
```bash
curl -X POST https://localhost:8000/auth/login -k -i -H "" -H "Content-Type: application/json" -d '{"user": "admin", "pass": "olia1234"}'
```
```json
HTTP/2 200 

{"session_id":"MVWmFIets6px78NA7sDVeVO7f_NLWtBHwbD3vtPt8YljGeD-heF0eSPO7NOyuTKPOt8PZa1n73hsaDWJOhKvi_qSatIxeSmBQxNgWVWjQiPiw8d0WoW9rlRF_qtn_3FWUvdnrYKYF0U-wHMDBZZqbHsLxre4KGXvjx-mAHNQUWg=","user":{"name":"admin","department":"IT","roles":["USER"]}}
```

#### private endpoint - again
```bash
curl https://localhost:8000/private -k -i -H "Authorization: bearer MVWmFIets6px78NA7sDVeVO7f_NLWtBHwbD3vtPt8YljGeD-heF0eSPO7NOyuTKPOt8PZa1n73hsaDWJOhKvi_qSatIxeSmBQxNgWVWjQiPiw8d0WoW9rlRF_qtn_3FWUvdnrYKYF0U-wHMDBZZqbHsLxre4KGXvjx-mAHNQUWg="
```
```bash
HTTP/2 200 
```
#### keep alive
To mark a session as in use for inactivity timeout.
```bash
curl -X POST https://localhost:8000/auth/kep-alive -k -i -H "Authorization: bearer MVWmFIets6px78NA7sDVeVO7f_NLWtBHwbD3vtPt8YljGeD-heF0eSPO7NOyuTKPOt8PZa1n73hsaDWJOhKvi_qSatIxeSmBQxNgWVWjQiPiw8d0WoW9rlRF_qtn_3FWUvdnrYKYF0U-wHMDBZZqbHsLxre4KGXvjx-mAHNQUWg="
```
```bash
HTTP/2 200 
```
#### logout
```bash
curl -X POST https://localhost:8000/auth/logout -k -i -H "Authorization: bearer MVWmFIets6px78NA7sDVeVO7f_NLWtBHwbD3vtPt8YljGeD-heF0eSPO7NOyuTKPOt8PZa1n73hsaDWJOhKvi_qSatIxeSmBQxNgWVWjQiPiw8d0WoW9rlRF_qtn_3FWUvdnrYKYF0U-wHMDBZZqbHsLxre4KGXvjx-mAHNQUWg="
```
```bash
HTTP/2 200 
```
#### clean exaple
```bash
docker compose down --rmi all
```

---
### License

Copyright © 2024, [Airenas Vaičiūnas](https://github.com/airenas).
Released under [The 3-Clause BSD License](LICENSE).

---
