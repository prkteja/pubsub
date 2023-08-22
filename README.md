# pubsub
A simple multi-client multi-channel message passing service

Can be used as a backend for chat applications, notification systems, etc
## Usage
- The main service exposes 3 REST endpoints to create a channel, get a list of channels, and attach as a client
- The attach endpoint takes a list of channels and the client role (Publisher/Subscriber)
- A successful attach request is upgraded to a websocket connection which can be used to stream text messages

## API
### Create new channel
**POST**  -  `/createChannel`

QueryParams:
- `name`: String (channel name)
- `size`: Int (channel buffer size)
   > optional, default is 32 messages


### Get all channels
**GET** - `/getChannels`

### Attach
**WS** - `/attach`

QueryParams:
- `channels`: String (comma seperated channel-id list)
- `role`: String (`publisher` or `subscriber`)

## Notes
- This does not provide any persistent storage and fault tolerance
- Long messages might lock the channel and force other pulishers to wait
