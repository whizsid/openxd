# Web Socket Authentication

Guarding the websocket is a critical thing in our application. Because we are dedicating a separate
thread per a websocket connection. So if we allowed anonymous users to connect websocket, then anyone
can call it multiple times and break our server. So we have to authenticate the user before they
starting the connection.

## Initializing the websocket anonymously and authenticate using a message

We can easily implement an authentication using the usual message passing functionality. But we have to
spawn a separate thread before authentication. So anyone can execute millions of requests per seconds.
Then our server will be crashed due to high amount of threads.

## Passing the JWT token with websocket request and validate

Browser is sending a HTTP method to initiate a WebSocket connection. So we can send the JWT token as a
`Bearer` token. But due to limitations in [JS WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
we can not pass any additional headers using the browser. But we can pass the JWT as a query string.
Passing the JWT as a query string is also not a secure method. Because the URL is logging in proxies, load balancers
and other middlewares. JWT is a sensitive data and we shouldn't allow to store it in anywhere. So we can not
pass the JWT as a query string.

## Implement a one-time ticketing system

Once the user authenticated, we can generate a one-time ticket with a specific code for the user. Then we can
pass it to the websocket request as a query parameter. If the ticket code exposed for a man in middle, then they can
not use it again to initiate a websocket connection.
