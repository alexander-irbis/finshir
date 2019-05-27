<div align="center">
  <h1>finshir</h1>
  
  <a href="https://gitter.im/Gymmasssorla/finshir">
    <img src="https://img.shields.io/badge/chat-on%20gitter-pink.svg">
  </a>
  <a href="https://travis-ci.com/Gymmasssorla/finshir">
    <img src="https://travis-ci.com/Gymmasssorla/finshir.svg?branch=master">
  </a>
  <a href="https://github.com/Gymmasssorla/finshir/blob/master/LICENSE">
    <img src="https://img.shields.io/badge/license-GPLv3-blue.svg">
  </a>
  <a href="https://crates.io/crates/finshir">
    <img src="https://img.shields.io/badge/crates.io-v0.3.1-orange.svg">
  </a>
  <a href="https://semver.org">
    <img src="https://img.shields.io/badge/semver-follows-green.svg">
  </a>
  
  <img src="ICON.png" width="520px"><br>
  
  You are seeing a high-performant, coroutines-driven, and fully customisable implementation of [Low & Slow](https://www.cloudflare.com/learning/ddos/ddos-low-and-slow-attack/) load generator designed for real-world pentesting. You can easily torify/proxify it using various platform-dependent utilities.
  
  <h4>
    <a href="https://github.com/Gymmasssorla/finshir/pulse">Pulse</a> &middot;
    <a href="https://github.com/Gymmasssorla/finshir/stargazers">Stargazers</a> &middot;
    <a href="https://github.com/Gymmasssorla/finshir/releases">Releases</a> &middot;
    <a href="https://github.com/Gymmasssorla/finshir/blob/master/CONTRIBUTING.md">Contributing</a>
  </h4>
</div>

----------

## Contents
 - [Demo](https://github.com/Gymmasssorla/finshir#demo)
 - [Advantages](https://github.com/Gymmasssorla/finshir#advantages)
 - [Disadvantages](https://github.com/Gymmasssorla/finshir#disadvantages)
 - [Installation](https://github.com/Gymmasssorla/finshir#installation)
   - [Building from crates.io](https://github.com/Gymmasssorla/finshir#building-from-cratesio)
   - [Building from sources](https://github.com/Gymmasssorla/finshir#building-from-sources)
   - [Pre-compiled binaries](https://github.com/Gymmasssorla/finshir#pre-compiled-binaries)
 - [Options](https://github.com/Gymmasssorla/finshir#options)
 - [Overview](https://github.com/Gymmasssorla/finshir#overview)
   - [Minimal command](https://github.com/Gymmasssorla/finshir#minimal-command)
   - [Test intensity](https://github.com/Gymmasssorla/finshir#test-intensity)
   - [Connections count](https://github.com/Gymmasssorla/finshir#connections-count)
   - [Custom data portions](https://github.com/Gymmasssorla/finshir#custom-data-portions)
   - [Logging options](https://github.com/Gymmasssorla/finshir#logging-options)
   - [TLS support](https://github.com/Gymmasssorla/finshir#tls-support)
   - [Generate a report](https://github.com/Gymmasssorla/finshir#generate-a-report)
 - [Contributing](https://github.com/Gymmasssorla/finshir#contributing)
 - [Legal disclaimer](https://github.com/Gymmasssorla/finshir#legal-disclaimer)
 - [Project references](https://github.com/Gymmasssorla/finshir#project-references)
 - [Contacts](https://github.com/Gymmasssorla/finshir#contacts)

----------

## Demo
<div align="center">
  <img src="https://github.com/Gymmasssorla/finshir/blob/master/DEMO.gif">
</div>

----------

## Advantages
 - **Coroutines-driven.** Finshir uses [coroutines](https://en.wikipedia.org/wiki/Coroutine) (also called lightweight threads) instead of ordinary threads, which lets you open many more connections with fewer system resources.

 - **Generic.** Unlike other Low & Slow utilities, Finshir lets you transmit arbitrary data sets over the [TCP](https://en.m.wikipedia.org/wiki/Transmission_Control_Protocol) protocol. It may be partial HTTP headers, empty spaces, and so on.
 
 - **Written in Rust.** How you can see, all the logic is written completely in [Rust](https://www.rust-lang.org/), which means that it leverages bare-metal performance and high-level safety (no SIGSEGV, SIGILL, and other "funny" stuff).

----------

## Disadvantages
 - **Platform-dependent.** Like most of pentesting utilities, this project is developed, tested, and maintained for only UNIX-based systems. If you are a Windows user, you probably need a [virtual machine](https://en.wikipedia.org/wiki/Virtual_machine) or another computer with UNIX.

----------

## Installation

### Building from crates.io
```bash
$ cargo install finshir
```

### Building from sources
```bash
$ git clone https://github.com/Gymmasssorla/finshir.git
$ cd finshir
$ cargo build --release
```

### Pre-compiled binaries
The easiest way to run Finshir on your system is to download the pre-compiled binaries from the [existing releases](https://github.com/Gymmasssorla/finshir/releases), which doesn't require any external software (unlike the two previous approaches).

----------

## Options
```
finshir 0.3.1
Temirkhan Myrzamadi <gymmasssorla@gmail.com>
A coroutines-driven Low & Slow traffic sender, written in Rust

USAGE:
    finshir [FLAGS] [OPTIONS] --receiver <SOCKET-ADDRESS>

FLAGS:
    -h, --help       Prints help information
        --use-tls    Use a TLS connection instead of the ordinary TCP protocol. It might be used to test HTTPS-based
                     services.
    -V, --version    Prints version information

OPTIONS:
        --connect-periodicity <TIME-SPAN>    This option will be applied if a socket connection error occurs (the next
                                             connection will be performed after this periodicity) [default: 7secs]
        --connect-timeout <TIME-SPAN>        Try connect a socket within a specified timeout. If a timeout is reached
                                             and a socket wasn't connected, the program will retry the operation later
                                             [default: 10secs]
    -c, --connections <POSITIVE-INTEGER>     A number of connections the program will handle simultaneously. This option
                                             also equals to a number of coroutines [default: 1000]
        --date-time-format <STRING>          A format for displaying local date and time in log messages. Type `man
                                             strftime` to see the format specification [default: %X]
        --failed-count <POSITIVE-INTEGER>    A number of failed data transmissions used to reconnect a socket to a
                                             remote web server [default: 5]
        --ip-ttl <UNSIGNED-INTEGER>          Specifies the IP_TTL value for all future sockets. Usually this value
                                             equals a number of routers that a packet can go through
        --json-report <FILENAME>             A file to which a JSON report (also called a "total summary") will be
                                             generated before exiting
    -f, --portions-file <LOCATION>           A file which consists of a custom JSON array of data portions, specified as
                                             strings.
                                             
                                             When a coroutine finished sending all portions, it reconnects its socket
                                             and starts sending them again.
    -r, --receiver <SOCKET-ADDRESS>          A receiver of generator traffic, specified as an IP address (or a domain
                                             name) and a port number, separated by a colon
    -d, --test-duration <TIME-SPAN>          A whole test duration, after which all spawned coroutines will stop their
                                             work [default: 64years 64hours 64secs]
        --text-report <FILENAME>             A file to which the program will generate a human-readable report (also
                                             called a "total summary") before exiting
    -v, --verbosity <LEVEL>                  Enable one of the possible verbosity levels. The zero level doesn't print
                                             anything, and the last level prints everything.
                                             
                                             Note that specifying the 4 and 5 verbosity levels might decrease
                                             performance, do it only for debugging. [default: 3]  [possible values: 0,
                                             1, 2, 3, 4, 5]
    -w, --wait <TIME-SPAN>                   A waiting time span before test execution used to prevent a launch of an
                                             erroneous (unwanted) test [default: 5secs]
        --write-periodicity <TIME-SPAN>      A time interval between writing data portions. This option can be used to
                                             modify test intensity [default: 30secs]
        --write-timeout <TIME-SPAN>          If a timeout is reached and a data portion wasn't sent, the program will
                                             retry the operation later [default: 10secs]
        --xml-report <FILENAME>              A file to which an XML report (also called a "total summary") will be
                                             generated before exiting

By default, Finshir generates 100 empty spaces as data portions. If you want to override this behaviour, consider using
the `--portions-file` option.

After test execution, you always receive a report (statistics about connections, transmissions, etc). If none of `--xml-
report`, `--text-report`, `--json-report` is specified, your terminal will be used.

For more information see <https://github.com/Gymmasssorla/finshir>.
```

----------

## Overview

### Minimal command
The following command spawns 1000 coroutines, each trying to establish a new TCP connection. When connections are established, it sends empty spaces every 30 seconds, thereby order a server to wait as long as it can:

```bash
# Specify one of the Google's IP addresses as a target web server
$ finshir --receiver=google.com:80
```

### Test intensity
Low & Slow techniques assume to be VERY SLOW, which means that you typically send a couple of bytes every N seconds. For instance, Finshir uses the 30 seconds interval by default, but it's modifiable as well:

```bash
# Test the Google's server sending data portions every one minute
$ finshir --receiver=google.com:80 --write-periodicity=1min
```

### Connections count
The default number of parallel connections is 1000. However, you can modify this limit using the `--connections` option, but be sure that you system is able to handle such amount of file descriptors:

```bash
# Modify the default limit of file descriptors to 17015
$ sudo ulimit -n 17015

# Test the target server using 17000 parallel TCP connections
$ finshir --receiver=google.com:80 --connections=17000
```

### Custom data portions
By default, Finshir generates 100 empty spaces as data portions to send. You can override this behaviour by specifying your custom messages as a file, consisting of a single JSON array. This example is focused on Google:

```bash
# Send partial HTTP headers to Google using `--portions-file`
$ finshir --receiver=google.com:80 --portions-file=files/google.json
```

### Logging options
Consider specifying a custom verbosity level from 0 to 5 (inclusively), which is done by the `--verbosity` option. There is also the `--date-time-format` option which tells Finshir to use your custom date-time format.

```bash
# Use a custom date-time format and the last verbosity level
$ finshir --receiver=google.com:80 --date-time-format="%F" --verbosity=5
```

### TLS support
Most of web servers today use the HTTPS protocol instead of HTTP, which is based on TLS. Since [v0.2.0](https://github.com/Gymmasssorla/finshir/releases/tag/v0.2.0), Finshir has functionality to connect through TLS using the `--use-tls` flag.

```bash
# Connect to the Google's server through TLS on 443 port (HTTPS)
$ finshir --receiver=google.com:443 --use-tls
```

### Generate a report
Report is a set of statistics variables like a total number of connections established, a total number of failed transmissions and so on. There is three options for this: `--xml-report`, `--json-report`, and `--text-report`:

```bash
# Test the Google's server and generate a JSON report at the end
$ finshir --receiver=google.com:80 --json-report=report.json
```

What means "at the end"? Well, Finshir will generate a report for you either if allotted time expires or if you cancel the process by Ctrl-C. You can look at the report examples in the [`/files`](https://github.com/Gymmasssorla/finshir/tree/master/files) folder:

([`files/report.json`](https://github.com/Gymmasssorla/finshir/blob/master/files/report.json))
```json
{
  "connections": {
    "failed": "0",
    "successful": "683",
    "total": "683"
  },
  "time": {
    "test-duration": "9s 897ms 561us 209ns",
    "test-start": "Mon, 27 May 2019 10:20:27 -0000"
  },
  "total-bytes-sent": "683",
  "total-errors": "0",
  "transmissions": {
    "failed": "0",
    "successful": "683",
    "total": "683"
  }
}
```

([`files/report.xml`](https://github.com/Gymmasssorla/finshir/blob/master/files/report.xml))
```xml
<?xml version="1.0" encoding="UTF-8"?>
<finshir-report>
  <total-bytes-sent>1534</total-bytes-sent>
  <total-errors>0</total-errors>
  <time>
    <test-start>Mon, 27 May 2019 10:18:57 -0000</test-start>
    <test-duration>38s 807ms 453us 842ns</test-duration>
  </time>
  <connections>
    <successful>1000</successful>
    <failed>0</failed>
    <total>1000</total>
  </connections>
  <transmissions>
    <successful>1534</successful>
    <failed>0</failed>
    <total>1534</total>
  </transmissions>
</finshir-report>
```

([`files/report.txt`](https://github.com/Gymmasssorla/finshir/blob/master/files/report.txt))
```
*********************** FINSHIR REPORT ***********************
Total bytes sent:         2000
Total errors:             0

Test start:               Mon, 27 May 2019 10:21:02 -0000
Test duration:            56s 29ms 777us 950ns

Successful connections:   1000
Failed connections:       0
Total connections:        1000

Successful transmissions: 2000
Failed transmissions:     0
Total transmissions:      2000
**************************************************************
```

If none of the options above has been specified, Finshir prints a report right to your terminal. That is, you can just run a test, cancel it later, and see the results which you can easily save. Perfect!

----------

## Contributing
You are always welcome for any contribution to this project! But before you start, you should read [the appropriate document](https://github.com/Gymmasssorla/finshir/blob/master/CONTRIBUTING.md) to know about the preferred development process and the basic communication rules.

----------

## Legal disclaimer
Finshir was developed as a means of testing stress resistance of web servers, and not for hacking, that is, the author of the project **IS NOT RESPONSIBLE** for any damage caused by your use of his program.

----------

## Project references
 - https://www.reddit.com/r/rust/comments/bm6ttn/finshir_a_coroutinesdriven_low_slow_ddos_attack/
 - https://www.producthunt.com/posts/finshir
 - https://www.reddit.com/r/hacking/comments/bpg0by/ive_written_a_customizable_optimized_alternative/
 - https://news.ycombinator.com/item?id=19931443
 - https://www.reddit.com/r/rust/comments/bpor6b/finshir_a_coroutinesdriven_and_fully_customizable/
 - https://news.ycombinator.com/item?id=19962333
 - https://www.reddit.com/r/rust/comments/bqyaok/finshir_v022_was_released_any_suggestions_to_low
 - https://www.reddit.com/r/rust/comments/btdu1a/finshir_v030_was_released_now_with_report/

----------

## Contacts
[Temirkhan Myrzamadi](https://github.com/Gymmasssorla) <[gymmasssorla@gmail.com](mailto:gymmasssorla@gmail.com)> (the author)
