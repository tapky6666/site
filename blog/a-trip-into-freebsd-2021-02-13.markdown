---
title: "A Trip into FreeBSD"
date: 2021-02-13
tags:
 - freebsd
---

I normally deal with Linux machines. Linux is what I know and it's what I've
been using since I was in college. A friend of mine has been coaxing me into
trying out [FreeBSD](https://www.freebsd.org), and I decided to try it out and
see what it's like. Here's some details about my experience and what I've
learned.

## Hardware

I've tried out FreeBSD on the following hardware:

- qemu/KVM on amd64
- Raspberry Pi 4 (4 GB)
- Raspberry Pi 3B (1 GB)

I've had the most luck with the Raspberry Pi 3 though. The KVM machine would
hang infinitely after the install process waiting for the mail service to do a
DNS probe of its own hostname (I do not host automagic FQDNS for my vms). I'm
pretty sure I was doing something wrong there but I wasn't able to figure out
what I needed to do in order to disable the DNS probe blocking startup.

[If you know what we were doing wrong here, please feel free to <a
href="/contact">contact</a> us with the thing we messed
up.](conversation://Mara/hacker)

After waiting for about 5 minutes I gave up and decided to try out the Raspberry
Pi 4. The Raspberry Pi 4 is the most powerful arm board I own. It has 4 GB of
ram and a quad core processor that is way more than sufficient for my needs. I
was hoping to use FreeBSD on that machine so I could benefit from the hardware
the most. Following the instructions on [the wiki
page](https://wiki.freebsd.org/arm/Raspberry%20Pi), I downloaded the 12.2 RPI
image and flashed it to an SD card using Etcher. I put the SD card in, turned
the raspi on and then waited for it to show up on the network.

Except it never showed up on the network. I ran scans with nmap (specifically
with the command `sudo nmap -sS -p 22 192.168.0.0/24`) and the IP address never
showed up. I also didn't see any new MAC addresses on the network, so that lead
me to believe that the pi was failing to boot. I downloaded an image for 13-BETA
and followed [this
guide](https://medium.com/swlh/freebsd-usb-boot-on-raspberry-pi-4-765cb6e75570)
that claims to make it work on the pi 4, but I got the same issue. The Raspberry
Pi 4 unfortunately has a micro-HDMI port on it, so I was unable to attach it to
my monitor to see any error messages. After trying for a while to see if I could
set up a serial port to get the serial log messages (spoiler: I couldn't), I dug
up my Pi 3 and stuck the same SD card into it, hooked it up to my monitor,
attached a spare keyboard to it and booted into FreeBSD first try.

## Using FreeBSD

FreeBSD is a very down to earth operating system. It also has a
[handbook](https://docs.freebsd.org/en/books/handbook/) that legitimately
includes all of the information you need to get up and running. Following the
handbook, I set a new password, installed the `pkg` tool, set up
[fish](https://fishshell.com) and then also installed the Go compiler toolchain
for the hell of it.

`pkg` is a very minimal looking package manager. It doesn't have very many
frills and it is integrated into the system pretty darn well. It looks like it
prefers putting everything into `/usr/local`, including init scripts and other
configuration files. 

This interestingly lets you separate out the concerns of the base system from
individual machine-local configuration. I am not sure if this also works with
files like `/etc/resolv.conf` or other system configuration files, but it does
really give `/usr/local` a reason to exist beyond being a legacy location for
yolo-installed software that may or may not be able to be upgraded separately.

## Custom Services

Speaking of services, I wanted to see how hard it would be to get a custom
service running on a FreeBSD box. At the minimum I would need the following:

- The binary built for freebsd/aarch64 and installed to `/usr/local/bin`
- A user account for that service
- An init script for that service
- To enable the init script in `/etc/rc.conf`

I decided to do this with a service I made years ago called
[whatsmyip](https://github.com/Xe/whatsmyip).

### Building a Binary

Building the service is easy, I just go into the directory and run `go build`.
Then I get a binary. Running it in another tmux tab, we can see it in action:

```console
$ curl http://[::1]:9090
::1
```

I can also run the curl command from my macbook:

```console
$ curl http://pai:9090
100.72.190.5
```

Cool, I've got a working service! Let's install it to `/usr/local/bin`:

```console
$ doas cp ./whatismyip /usr/local/bin
```

[Wait, `doas`? What is `doas`? It looks like it's doing something close to what
sudo does.](conversation://Mara/hmm)

[doas](https://en.wikipedia.org/wiki/Doas) is a program that does most of the
same things that sudo does, but it's a much smaller codebase. I decided to try
out doas for this install for no other reason than I thought it would be a cool
thing to learn. It's actually pretty simple, and I'm going to look at using it
elsewhere (with an alias for `sudo` -> `doas`).

### Service User

The handbook says that we use the
[adduser](https://people.freebsd.org/~blackend/en_US.ISO8859-1/books/handbook/users-synopsis.html)
command to add users to the system. So, let's run `adduser` to create a
`whatsmyip` user:

```console
# adduser
Username: whatsmyip
Full name: github.com/Xe/whatsmyip
Uid (Leave empty for default): 666
Login group [whatsmyip]:
Login group is whatsmyip. Invite whatsmyip into other groups? []:
Login class [default]:
Shell (sh csh tcsh bash rbash git-shell fish nologin) [sh]: sh
Home directory [/home/whatsmyip]: /var/db/whatsmyip
Home directory permissions (Leave empty for default):
Use password-based authentication? [yes]: no
Lock out the account after creation? [no]: yes
Username   : whatsmyip
Password   : <disabled>
Full Name  : github.com/Xe/whatsmyip
Uid        : 666
Class      :
Groups     : whatsmyip
Home       : /var/db/whatsmyip
Home Mode  :
Shell      : /bin/sh
Locked     : yes
OK? (yes/no): yes
adduser: INFO: Successfully added (whatsmyip) to the user database.
adduser: INFO: Account (whatsmyip) is locked.
Add another user? (yes/no): no
Goodbye!
```

It's a bit weird that there's not a flow for creating a "system user" that
automatically sets the flags that I expect from Linux system administration, but
I was able to specify the values manually without too much effort.

Something interesting is that when I set the user account to `nologin` I
actually was unable to log in as the user. Usually in Linux you can hack around
this with `su` flags but FreeBSD doesn't have this escape hatch. Neat.

### Init Script

Now that I had the service account set up, I need to write an init service that
will start this program on boot. Following other parts of the handbook I was
able to get a base script that looks like this:

```shell
#!/bin/sh
#
# PROVIDE: whatsmyip
# REQUIRE: DAEMON
# KEYWORD: shutdown

. /etc/rc.subr

name=whatsmyip
rcvar=whatsmyip_enable

command="/usr/sbin/daemon"
command_args="-S -u whatsmyip -r -f -p /var/run/whatsmyip.pid /usr/local/bin/whatsmyip"
load_rc_config $name

#
# DO NOT CHANGE THESE DEFAULT VALUES HERE
# SET THEM IN THE /etc/rc.conf FILE
#
whatsmyip_enable=${whatsmyip_enable-"NO"}
pidfile=${whatsmyip_pidfile-"/var/run/whatsmyip.pid"}

run_rc_command "$1"
```

Now I can copy this file to `/usr/local/etc/rc.d/whatsmyip` and then make sure
it's set to the permissions `0555` with something like:

```console
$ chmod 0555 ./whatsmyip.rc
$ doas cp ./whatsmyip.rc /usr/local/etc/rc.d/whatsmyip
```

### Enabling The Service

Once I had the file in the right place, I enabled the service in `/etc/rc.conf`
like this:

```shell
# whatsmyip
whatsmyip_enable="YES"
```

Then I started the service with `service whatsmyip start`, and I was unable to
start the service. I got this error:

```
Feb 13 20:40:00 pai freebsd[1519]: /usr/local/etc/rc.d/whatsmyip: WARNING: failed to start whatsmyip
```

And no other useful information to help me actually fix the problem. I assume
there's some weirdness going on with permissions, so let's sidestep the user
account for now and just run the service as root directly by changing the
`command_args` in `/usr/local/etc/rc.d/whatsmyip`:

```shell
command_args="-S -r -p /var/run/whatsmyip.pid /usr/local/bin/whatsmyip"
```

Restarting the service, everything works! I can hit that service all I want and
I get back the IP address that I used to hit that service. 

## What I Learned

FreeBSD has _excellent_ documentation. The people on the documentation team
really care about making the handbook useful. I wish it went into more detail
about best practices for making your own services (I had to crib from some other
service files as well as googling for a minimal template), but overall it gives
you enough information to get off the ground.

FreeBSD is also fairly weird. It's familiar-ish, but it's a very different
experience. It's also super-minimal. Looking at the output of `ps x`, there's
only 45 processes running on the system, including kernel threads.

```
root@pai ~# ps x | wc -l
      45
```

The only processes are `init`, `dhclient`, a device manager, `syslog`,
`tailscaled`, `sshd`, `cron`, `whatsmyip`, `fish` and a few instances of `getty`
to allow me to log in with an HDMI monitor and keyboard should I need to. That's
it. That's all that's running. It's only using 96 MB of ram and most of the
machine's power is left over to me.

It's just a shame that FreeBSD support for programming languages is so poor in
general. Go works fine on it, but Rust doesn't have any pre-built binaries for
the compiled (and using ports/pkg isn't an option because aarch64 is a tier-2
architecture in FreeBSD land which means that it's not guaranteed to have
prebuilt binaries for everything). Compiling Rust from source also really isn't
an option because I don't have enough ram on my raspi to do that. Go works
though.

I really wonder how this kind of network effect will boil down with more and
more security libraries like
[pyca](https://github.com/pyca/cryptography/issues/5771) integrating Rust deeper
into core security components. It probably means that people are going to have
to step up and actually do the legwork required to get Rust working on more
platforms, however it definitely is going to leave some older hardware or less
commonly used configurations (like aarch64 FreeBSD) in the dust if we aren't
careful. Maybe this isn't a technical problem, but it is definitely something
interesting to think about.

Overall, FreeBSD is an interesting tool and if I ever have a good use for it in
my server infrastructure I will definitely give it a solid look. I just wish it
was as easy to manage a FreeBSD system as it is to manage a NixOS system. A lot
of my faffing about with `rc.conf` and rc scripts wouldn't have needed to
happen if that was the case.
