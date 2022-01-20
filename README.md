# comet 

A simple **Com**munication **E**avesdropping **T**ool.

`comet` is a tool I developed to fit some of my needs while performing hardware security assessments. 
Its main goal is to help sniffing UART communications, for instance between a microcontroller and a modem. 

`comet` will automatically save the captured data in two formats: 
1. The raw line data as received 
2. The individual lines captured in a JSON file (the raw bytes are encoded to a hex string)

By default, it will try to decode the data before displaying it to the console. 
If it fails to do so, the data is encoded to a hex string before being displayed.

```raw
[*] session: comet_20220206_185814
[*] port 1: /dev/tty.usbmodem
[*] baudrate for port 1: 115200
[*] start listening on port 1...
[<][2022-02-06 18:58:14.245977 UTC] Hello world!
[<][2022-02-06 18:58:14.622941 UTC] deadbeef
[<][2022-02-06 18:58:14.661773 UTC] Hello world!
[<][2022-02-06 18:58:15.037780 UTC] deadbeef
[<][2022-02-06 18:58:15.076815 UTC] Hello world!
[<][2022-02-06 18:58:15.452969 UTC] deadbeef
```

`comet` can also handle two different "sources" (port 1 and 2) and will multiplex them (Note: be aware there is no guaranty on the order). 
This allows to sniff on both RX and TX simultaneously.

## Simple usage

In its simplest usage (listening on a port with all default parameters):

```bash
$ comet -p /dev/tty.port
```

This will listen on `/dev/tty.port` with a baudrate of 115200 bd/s, and will save the two resulting files in a folder `comet_yyyymmdd_hhmmss`.

## Usage

```bash
USAGE:
    comet [OPTIONS]

OPTIONS:
        --baud <baud>                    Baudrate for port [default: 115200]
        --baud2 <baud2>                  Baudrate for port 2 [default: 115200]
        --common-baudrates               List common baudrates
    -h, --help                           Print help information
        --list-ports                     List available TTY ports
        --no-colour                      Do not display colours
        --no-direction                   Do not display message direction information
        --no-timestamp                   Do not display timestamps
    -p, --port <port>                    Port 1
        --port2 <port2>                  Port 2
        --session-name <session-name>    Name of the capture session
    -V, --version                        Print version information
```

## Additional functionalities

`comet` comes with two little "utils": one to *list the available ports* and one to *display a table of the most common baudrates*.
I have added those functionalities because I often found myself having to do it during an assessment (for instance when the baudrates is *not known*, but the bit duration is).

### List available port

```bash
$ comet --list-ports
```

### Common baudrates

```bash
$ comet --common-baudrates
```

Result:
```
$ comet --common-baudrates
+--------+--------------+
| Bauds  | Bit duration |
+--------+--------------+
| 50     | 20.000 ms    |
+--------+--------------+
| 75     | 13.333 ms    |
+--------+--------------+
| 110    | 9.091 ms     |
+--------+--------------+
...
```

## Contributing

Any contributions to the tool are welcome!

Writing this tool was also for me a way to try out rust and as such, any contributions to the code quality (to make it more rustacean for instance) are more than welcome!
