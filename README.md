# Webhook-Server

[![GitHub release](https://img.shields.io/github/tag/nukesor/webhook-server.svg)](https://github.com/nukesor/webhook-server/releases/latest)
[![Actions Status](https://github.com/Nukesor/webhook-server/workflows/Tests/badge.svg)](https://github.com/Nukesor/webhook-server/actions)
 [![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![Paypal](https://github.com/Nukesor/images/blob/master/paypal-donate-blue.svg)](https://www.paypal.me/arnebeer/)
[![Patreon](https://github.com/Nukesor/images/blob/master/patreon-donate-blue.svg)](https://www.patreon.com/nukesor)


![Webhook server example](https://github.com/Nukesor/images/blob/master/webhook-server.gif)

Webhook server is a minimal footprint server to execute stuff on your server on incoming http requests.
It has been designed for continuous integration and supports Github's webhooks out of the box.

Webhook server also comes with a custom scheduler. which by default prevents parallel deployments and unnecessary deployment executions.
In case you want to queue many parallel load-heavy long-running tasks, it allows you to specify the amount of concurrent tasks for each type, to prevent overburdening your system.
Tasks can be processed in parallel or one-by-one, the mode of execution and amount of parallel processes can be configured per webhook type.

Take a look at the example config file [webhook_server.yml](https://github.com/Nukesor/webhook-server/blob/master/webhook_server.yml).

**DISCLAIMER:**

This project is relatively young, which means: 
- Cross compilations for mac isn't tested yet.
- Cross compilations for windows isn't tested yet.
- There are probably some things regarding paths that aren't right on those platforms yet


## Installation:
**Arch Linux:**  
Just install it with `yay -S webhook-server-git` or (yaourt if you like)

**Releases:**  
Each release includes prebuild binaries for Linux, Mac and Windows.
You can finde them in the `releases` tab of the project.


**Git installation:**

    git clone https://github.com/nukesor/webhook-server
    cd webhook-server
    cargo build --release
    cp target/release/webhookserver /bin/webhookserver


## Configuration: 

Webhook-Server is configured via files in this order:

- `/etc/webhook_server.yml`
- `~/.config/webhook_server.yml`
- `./webhook_server.yml`

Config values of higher hierarchy config files are overwritten by lower hierarchy config files. E.g. a value in `/etc/webhook_server.yml` can be overwritten by `~/.config/webhook_server.yml`.

Mac-OS:
- `~/Library/Application Support/webhook_server.yml`
- `~/Library/Preferences/webhook_server.yml`
- `./webhook_server.yml`

Windows: 
- `$APPDATA$\Roaming\webhook_server\webhook_server.yml`
- `.\webhook_server.yml`

### Config values
- `domain (127.0.0.1)` The domain the server should listen on
- `port (8000)` The port the server should listen on
- `ssl_private_key (null)` Path to SSL private key. The server will use it's own ssl certificate. Recommended, if you aren't using a proxy webserver, that already uses SSL. Using any kind of SSL is highly recommended, especially if you publicly expose your endpoint.
- `ssl_cert_chain (null)` Path to SSL cert. Also required for SSL setup.
- `workers (4)` The amount of workers for parallel webhook processing. If you plan on processing a LOT of requests or triggering long running task, increase the worker count.
- `basic_auth_user (null)` Your user if you want to do basic auth. Check the `Building a request` section for more information on basic_auth headers
- `basic_auth_password (null)` Your password if you want to do basic auth.
- `secret (null)` A secret for authentication via payload signature verification. Check the `Building a request` section for more information on signature headers. Can be, for instance, be created with `pwgen 25 1`
- `basic_auth_and_secret (false)` By default it's only required to authenticate via BasicAuth OR signature authentication. If you want to be super safe, set this to true to require both.
- `webhooks` A list of webhooks. The whole thing looks pretty much like this:

```
webhooks:
  -
    name: 'ls'
    command: '/bin/ls {{param1}} {{param2}}'
    cwd: '/home/user'
```

**Webhook config values**
- `name` The name of the webhook, also the endpoint that's used to trigger the webhooks. E.g. `localhost:8000/ls`.
- `command` The command thats actually used. If you want to dynamically build the command, you can use templating parameters like `{{name_of_parameter}}`.
- `cwd` The current working directory the command should be executed from.
- `mode (deploy)` Determines the mode at which the command shall be executed.
    1. `deploy` At most one queued AND at most one running. This is the default.
    2. `single` At most one queued OR running Item per webhook type
    3. `parallel` Unlimited queued and a default of max 4 parallel tasks. The number can be adjusted.
- `parallel_processes (4)` The max amount of parallel tasks when running in `parallel` mode.


## Misc files

There are some template files for your setup in the [misc folder](https://github.com/Nukesor/webhook-server/tree/master/misc) of the repository.
These include:

- A nginx proxy route example
- A systemd service file

If you got anything else that might be useful to others, feel free to create a PR.


## Github Webhook Setup:

Go to your project's settings tab and select webhooks. Create a new one and set these options:

- Content-Type: Json
- Secret: Same string as in your config
- Enable SSL verification: Recommended, if you have any kind of SSL
- Just the push event (The payload isn't used anyway)


You can click on the `Recent Deliveries` to redeliver any sent webhook, in case you want to debug your setup.


## Building a request

Webhook server accepts JSON POST requests and simple GET requests.

This is an example POST request issued with `httpie` and a secret of `72558847d57c22a2f19d711537cdc446` and `test:testtest` basic auth credentials:

```
echo -n '{"parameters":{"param1":"-al","param2":"/tmp"}}' | http POST localhost:8000/ls \
        Signature:'sha1=d762407ca7fb309dfbeb73c080caf6394751f0a4' \
        Authorization:'Basic dGVzdDp0ZXN0dGVzdA=='
```


If you don't need templating, you can send a simple GET request:

```
http GET localhost:8000/ls Authorization:'Basic dGVzdDp0ZXN0dGVzdA=='
```


**Payload:**

The payload is a simple JSON object, with a single entry `parameters`.
This object contains all parameters necessary for rendering the template.
If no templating is needed, you can provide an empty object as payload or simply call the route via `GET`.

For instance, the payload for the command `'/bin/ls {{param1}} {{param2}}'` could look like this:

```
{
    "parameters": {
        "param1": "-al",
        "param2": "/tmp"
    }
}
```

This would result in the execution of `ls -al /tmp` by the server.


**Headers:**

- `Authorization`: If `basic_auth_username` and `basic_auth_password` is specified, this should be the standard `Basic` base64 encoded authorization header. [Basic Auth guide](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization)
- `Signature:` If you specify a secret, the content of the signature is the HMAC of the json payload with the UTF8-encoded secret as key.
    This procedure is based on Github's webhook secret system. (Github tells you to use a hex key, but they interpret it as UTF8 themselves -.-)  
    Python example: `hmac.new(key, payload, hashlib.sha1)`  
    Ruby example: `OpenSSL::HMAC.hexdigest("SHA1", key, payload)`  
    [Github guide](https://developer.github.com/webhooks/securing/) 
- `X-Hub-Signature`: If there is no `Signature`, this header will be used for the signature check (to support Github's webhooks).


## Query current status

You can get the current state of the webhook scheduler and finished tasks by querying the root (`/`) of the server.
This will give you a JSON response with information about pretty much everything going on right now.

To access the route, authenticate via `Basic` authorization.
If no `Basic` authorization is specified while a secret exists, the secret will be used with an empty body.
In case no authentication is used at all, the status can be queried by anyone. Please use some kind of authentication.


## Security

**Code injection:**
When compiling dynamic commands with templating, you make yourself vulnerable to code injection, since the compiled commands are executed by the system shell.
If you plan on using templating and publicly exposing your service, please use some kind of authentication.

1. You can use a secret to verify the payload with a signature (Github's authentication method). Anyway, this method is a bit annoying to implement, if you write your own implementation.
2. You can use basic auth.
3. If you want to be super safe, you can require both authentication methods.


**SSL:**
Especially when using Basic Auth or templating it's highly recommended to use SSL encryption.
This can be either done by your proxy web server (nginx, apache, caddy) or directly in the application.
Otherwise your credentials or your template payload could leak to anybody listening.

An example cert and key can be created like this `openssl req -nodes -new -x509 -keyout test.pem -out test.pem`.  
If you need a password input for the private key, please create an issue or PR (much appreciated).
