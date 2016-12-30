**Disclaimer: This is a draft version**

# rori_desktop_endpoint

This is a simple endpoint for _[RORI](https://github.com/AmarOk1412/rori/)_. Just a simple application to execute shell and music commands from `rori_server` on a linux computer.

# Installation

This application requires _Rust language_ (please read this [page](https://www.rust-lang.org/en-US/install.html)) and _openssl_. To build the software, you just have to launch `cargo build` in this repository. You will need _Python 3_ and _rhythmbox_ for the music script (or you will need to rewrite _scripts/music.py_)

# Configuration

## Connect to rori_server

Note: you need to configure a `rori_server` first.

### Entry point side

You can configure the connection from _config_server.json_:

```json
"rori_ip":"ip of rori_server",
"rori_port":"port of rori_server",
"name":"the name you want",
"secret":"the secret you want",
```

### rori_server side

You need to authorize the entrypoint to communicate with `rori_server`. In _config_server.json_ you need to add:

```json
"authorize": [
  {
    "name":"the name you choose",
    "secret":"sha256 of the secret you choose"
  }
]
```

## Authorize rori_server

You need to authorize `rori_server` to communicate with you. In _config_server.json_ you must add:

```json
"authorize": [
  {
    "name":"the name of the rori_server",
    "secret":"sha256 of the secret of the rori_server"
  }
]
```

## Tls configuration

All connections need to be secured. So you need to generate a private key and a certificate. On linux, you can run this following command: `openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem`. It will create a certificate (_cert.pem_) and a private key (_key.pem_). Now, you can add these files to _config_server.json_.

## Final

The final _config_server.json_ should look like this:

```json
{
 "ip":"0.0.0.0",
 "port":"1414",
 "rori_ip":"127.0.0.1",
 "rori_port":"1412",
 "owner":"AmarOk",
 "name":"rori_desktop_client",
 "compatible_types":"music|shell",
 "cert":"key/cert.pem",
 "key":"key/key.pem",
 "secret":"secret",
 "authorize": [
   {
     "name":"rori_server",
     "secret":"2BB80D537B1DA3E38BD30361AA855686BDE0EACD7162FEF6A25FE97BF527A25B"
   }
 ]
}
```

# Command execution

## shell commands

When this endpoint receives a RORIData from `rori_server` with _datatype = shell_ it executes `sh -c "RORIData.content"`. If you want to know what shell commands you can receive, you just have to configure modules of your `rori_server`.

## music commands

When this endpoint receives a RORIData from `rori_server` with _datatype = music_ it executes `python3 scripts/music.py "RORIData.content"`. For now, the script is pretty basic:

```python
import sys
import subprocess

if len(sys.argv) > 1:
    if sys.argv[1] == "start":
        print("start music")
        subprocess.Popen("rhythmbox \"`find ~/Music/*.mp3 -type f | shuf -n 1`\"&", shell=True)
    if sys.argv[1] == "stop":
        print("stop music")
        subprocess.Popen("pkill rhythmbox", shell=True)
```

So, you can rewrite this script to execute what you want.

# Execution

A binary is present in the _target/_ directory after a `cargo build` or you can execute `cargo run` in your shell.

# License

```
DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
        Version 2, December 2004

Copyright (C) 2016 Sébastien (AmarOk) Blin <https://enconn.fr>

Everyone is permitted to copy and distribute verbatim or modified
copies of this license document, and changing it is allowed as long
as the name is changed.

DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

0\. You just DO WHAT THE FUCK YOU WANT TO.
```

# Contribute

Please, feel free to contribute to this project in submitting patches, corrections, opening issues, etc.
