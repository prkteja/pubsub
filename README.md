# pubsub
A simple multi-client multi-channel message passing service

Can be used as a backend for chat applications, notification systems, etc
### Usage
- The main service exposes 3 REST endpoints to create a channel, get a list of channels, and attach as a client
- The attach endpoint takes a list of channels and the client role (Publisher/Subscriber)
- A successful attach request is upgraded to a websocket connection which can be used to stream text messages
