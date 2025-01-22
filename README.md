## Twilio Webhook: Rust server that persists SMS in Redis

### Motivation

I recently upgraded to a global mobile plan, so I gave up on my T-Mobile US number, but I'd still like to be able to receive messages from my US friends when I'm in town, so I got a Twilio US number and wanted to persist SMSs I get to that number somewhere. 

As of now, this project is a simple Twilio webhook that persists all messages sent to my Twilio number in a Redis instance in plain text (no attachment/media support).

### Install

Rename `.env.template` to `.env` and fill in the values. There's a Dockerfile if you need to spin a Redis instance for dev,

```sh
./docker-compose up -d --build
```

should get that covered.

Run the project with

```sh
cargo run
```

### Test in dev

```sh

curl --location 'http://localhost:{PORT}/incoming?token={PRIVATE_EXCHANGE_TOKEN}' \
--header 'Content-Type: application/x-www-form-urlencoded' \
--data-urlencode 'From=32893829389' \
--data-urlencode 'To=12334429389' \
--data-urlencode 'Body=Hey there' \
--data-urlencode 'AccountSid=3232323' \
--data-urlencode 'MessageSid=38929381208392893'
```

### Test in prod

Go to your Twilio phone number dashboard and set the webhook URL to this project's URL in production:

![Screenshot 2024-02-22 at 18 37 11](https://github.com/spiceee/us-sms-gateway/assets/12278/198343c0-7e94-4534-8d7a-086a3977f049)
