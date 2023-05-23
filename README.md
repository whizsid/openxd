# OpenXD - Let's make open source UIs

This is a work in progress project to bring a open world to UI/UX
designers.

## Sub crates

- `transport` - Requests and Response types that share between backend and
frontend
- `ui` - Platform agnostic UI logics.
- `app` - Platform agnostic application logics
- `web` - Web frontend
- `server` - Web socket server that handling active sessions
- `standalone` - Standalone application

## Architecture

```
+-------------+                                            
|             <--------------+                             
|     Web     |              |                             
|             |       +------|------+                      
+-------------+       |             <--------------+       
       +---------------     UI      |              |       
+------v------+       |             |       +------|------+
|             |       +-------------+       |             |
| Standalone  |                             |  Transport  |
|             |       +-------------+       |             |
+------^------+       |             |       +------|------+
       +---------------     App     |              |       
+-------------+       |             <--------------+       
|             |       +------|------+                      
|   Server    |              |                             
|             <--------------+                             
+-------------+                                            
```

## Status

- [x] Make standalone application, web server, web frontend by sharing same source code.
- [x] 'Open File' implementation.
- [ ] `New` implementation.
