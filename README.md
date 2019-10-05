# Webhook-Server

# WARNING This is still under heavy active development.
**The documentation is mainly a roadmap for myself. not everything works as described yet.**

Webhook server is a minimal footprint server to execute stuff on your server on POST requests.
It's easy to setup and super useful for custom CI. It supports Github's webhook secrets out of the box.

## Configuration: 

Configuration is done via configuration files in this order.

- `/etc/webhook_server.yml`
- `~/.config/webhook_server.yml`
- `./webhook_server.yml`

Config values of higher hierarchy config files are overwritten by lower hierarchy config files. E.g. a value in `/etc/webhook_server.yml` can be overwritten by `~/.config/webhook_server.yml`.

An example config file can be found in the `webhook_server.yml` file of the repository.

### Config values
- `domain (127.0.0.1)` The domain the server should listen on
- `port (8000)` The port the server should listen on
- `ssh_cert (null)` The server serve directly using it's own ssl certificate. Recommended, if not using a proxy webserver, which already uses SSL. Using any kind of SSL is highly recommended, if you publicly expose your endpoint.
- `workers (4)` The amount of workers for parallel webhook processing. If you plan on processing a LOT of requests or triggering long running task, increase the worker count.
- `basic_auth_user (null)` Your user if you want to do basic auth. Check the `Building a request` section for more information on basic_auth headers
- `basic_auth_password (null)` Your password if you want to do basic auth.
- `secret (null)` A secret for authentication via payload signature verification. Check the `Building a request` section for more information on signature headers. Can be, for instance, be created with `pwgen 25 1`
- `basic_auth_and_secret (false)` By default it's only required to authenticate via BasicAuth OR signature authentication. If you want to be super safe, set this to true to require both.
- `webhooks` A list of webhooks. Such a webhook looks like this:

```
  -
    name: 'ls'
    command: '/bin/ls {{param1}} {{param2}}'
    cwd: '/home/user'
```

**Webhook config values**
- `name` The name of the webhook, also the endpoint that's used to trigger the webhooks. E.g. `localhost:8000/ls`
- `command` The command thats actually used. If you want to dynamically build the command, you can use templating parameters like `{{name_of_parameter}}`.
- `cwd` The current working directory the command should be executed from.

## Buildling a request

Webhook server accepts json encoded POST requests.  
This is an example request issued with `httpie` and a secret of `72558847d57c22a2f19d711537cdc446`:

```
echo -n '{"parameters":{"param1":"-al","param2":"/tmp"}}' | http POST localhost:8000/ls \
        Signature:'sha1=d762407ca7fb309dfbeb73c080caf6394751f0a4' \
        Authorization:'Basic d2ViaG9vazp0aGlzaXNhcGFzc3dvcmQ='
```

**Payload:**

The payload is a simple JSON encoded dictionary, with a single entry `parameters`, which contains a dictionary with all parameters necessary for rendering the template.
If no templating is necessary, the payload can be discarded.

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

- `Authorization`: If `basic_auth.username` and `basic_auth.password` is specified, this should be the standard `base64` encoded authorization header.
- `Signature:` If you specify a secret, the content of the signature is the HMAC of the json payload with the UTF8-encoded secret as key.
    This procedure is based on Github's webhook secret system. (Github says to use a hex key, but they interpret it as UTF8 -.-)
    Python example: `hmac.new(key, payload, hashlib.sha1)`  
    Ruby example: `OpenSSL::HMAC.hexdigest("SHA256", key, payload)`
- `X-Hub-Signature`: This is the default of Github's webhooks and is a fallback, if `Signature` is not specified.



## Security

**Code injection:**

If you compile dynamic commands with templating, you make yourself vulnerable to code injection, since the compiled commands are executed by the system shell.
It's always a good idea to use a secret, but if you plan on using templating and publicly exposing your service, **please to use basic auth, secrets or even better both**.
