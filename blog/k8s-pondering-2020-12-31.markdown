---
title: Kubernetes Pondering
date: 2020-12-31
tags:
 - k8s
 - kubernetes
 - soyoustart
 - kimsufi
 - digitalocean
 - vultr
---

Right now I am using a freight train to mail a letter when it comes to hosting
my web applications. If you are reading this post on the day it comes out, then
you are connected to one of a few replicas of my site code running across at
least 3 machines in my Kubernetes cluster. This certainly _works_, however it is
not very ergonomic and ends up being quite expensive.

I think I made a mistake when I decided to put my cards into Kubernetes for my
personal setup. It made sense at the time (I was trying to learn Kubernetes and
I am cursed into learning by doing), however I don't think it is really the best
choice available for my needs. I am not a large company. I am a single person
making things that are really targeted for myself. I would like to replace this
setup with something more at my scale. Here are a few options I have been
exploring combined with their pros and cons.

Here are the services I currently host on my Kubernetes cluster:

- [this site](/)
- [my git server](https://tulpa.dev)
- [hlang](https://h.christine.website)
- A few personal services that I've been meaning to consolidate
- The [olin demo](https://olin.within.website/)
- The venerable [printer facts server](https://printerfacts.cetacean.club)
- A few static websites
- An IRC server (`irc.within.website`)

My goal in evaluating other options is to reduce cost and complexity. Kubernetes
is a very complicated system and requires a lot of hand-holding and rejiggering
to make it do what you want. NixOS, on the other hand, is a lot simpler overall
and I would like to use it for running my services where I can.

Cost is a huge factor in this. My Kubernetes setup is a money pit. I want to
prioritize cost reduction as much as possible.

## Option 1: Do Nothing

I could do nothing about this and eat the complexity as a cost of having this
website and those other services online. However over the year or so I've been
using Kubernetes I've had to do a lot of hacking at it to get it to do what I
want. 

I set up the cluster using Terraform and Helm 2. Helm 3 is the current
(backwards-incompatible) release, and all of the things that are managed by Helm
2 have resisted being upgraded to Helm 3.

I'm going to say something slightly controversial here, but YAML is a HORRIBLE
format for configuration. I can't trust myself to write unambiguous YAML. I have
to reference the spec constantly to make sure I don't have an accidental
Norway/Ontario bug. I have a Dhall package that takes away most of the pain,
however it's not flexible enough to describe the entire scope of what my
services need to do (IE: pinging Google/Bing to update their indexes on each
deploy), and I don't feel like putting in the time to make it that flexible.

[This is the regex for determining what is a valid boolean value in YAML:
`y|Y|yes|Yes|YES|n|N|no|No|NO|true|True|TRUE|false|False|FALSE|on|On|ON|off|Off|OFF`.
This can bite you eventually. See the <a
href="https://hitchdev.com/strictyaml/why/implicit-typing-removed/">Norway
Problem</a> for more information.](conversation://Mara/hacker)

I have a tor hidden service endpoint for a few of my services. I have to use an
[unmaintained tool](https://github.com/kragniz/tor-controller) to manage these
on Kubernetes. It works _today_, but the Kubernetes operator API could change at
any time (or the API this uses could be deprecated and removed without much
warning) and leave me in the dust.

I could live with all of this, however I don't really think it's the best idea
going forward. There's a bunch of services that I added on top of Kubernetes
that are dangerous to upgrade and very difficult (if not impossible) to
downgrade when something goes wrong during the upgrade.

One of the big things that I have with this setup that I would have to rebuild
in NixOS is the continuous deployment setup. However I've done that before and
it wouldn't really be that much of an issue to do it again.

NixOS fixes all the jank I mentioned above by making my specifications not have
to include the version numbers of everything the system already provides. You
can _actually trust the package repos to have up to date packages_. I don't 
have to go around and bump the versions of shims and pray they work, because
with NixOS I don't need them anymore.

## Option 2: NixOS on top of SoYouStart or Kimsufi

This is a doable option. The main problem here would be doing the provision
step. SoYouStart and Kimsufi (both are offshoot/discount brands of OVH) have
very little in terms of customization of machine config. They work best when you
are using "normal" distributions like Ubuntu or CentOS and leave them be. I
would want to run NixOS on it and would have to do several trial and error runs
with a tool such as [nixos-infect](https://github.com/elitak/nixos-infect) to
assimilate the server into running NixOS.

With this option I would get the most storage out of any other option by far. 4
TB is a _lot_ of space. However, SoYouStart and Kimsufi run decade-old hardware
at best. I would end up paying a lot for very little in the CPU department. For
most things I am sure this would be fine, however some of my services can have
CPU needs that might exceed what second-generation Xeons can provide.

SoYouStart and Kimsufi have weird kernel versions though. The last SoYouStart
dedi I used ran Fedora and was gimped with a grsec kernel by default. I had to
end up writing [this gem of a systemd service on
boot](https://github.com/Xe/dotfiles/blob/master/ansible/roles/soyoustart/files/conditional-kexec.sh)
which did a [`kexec`](https://en.wikipedia.org/wiki/Kexec) to boot into a
non-gimped kernel on boot. It was a huge hack and somehow worked every time. I
was still afraid to reboot the machine though.

Sure is a lot of ram for the cost though.

## Option 3: NixOS on top of Digital Ocean

This shares most of the problems as the SoYouStart or Kimsufi nodes. However,
nixos-infect is known to have a higher success rate on Digital Ocean droplets.
It would be really nice if Digital Ocean let you upload arbitrary ISO files and
go from there, but that is apparently not the world we live in.

8 GB of ram would be _way more than enough_ for what I am doing with these
services.

## Option 4: NixOS on top of Vultr

Vultr is probably my top pick for this. You can upload an arbitrary ISO file,
kick off your VPS from it and install it like normal. I have a little shell
server shared between some friends built on top of such a Vultr node. It works
beautifully.

The fact that it has the same cost as the Digital Ocean droplet just adds to the
perfection of this option.

## Costs

Here is the cost table I've drawn up while comparing these options:

| Option        | Ram                | Disk                                  | Cost per month  | Hacks        |
| :---------    | :----------------- | :------------------------------------ | :-------------- | :----------- |
| Do nothing    | 6 GB (4 GB usable) | Not really usable, volumes cost extra | $60/month       | Very Yes     |
| SoYouStart    | 32 GB              | 2x2TB SAS                             | $40/month       | Yes          |
| Kimsufi       | 32 GB              | 2x2TB SAS                             | $35/month       | Yes          |
| Digital Ocean | 8 GB               | 160 GB SSD                            | $40/month       | On provision |
| Vultr         | 8 GB               | 160 GB SSD                            | $40/month       | No           |

I think I am going to go with the Vultr option. I will need to modernize some of
my services to support being deployed in NixOS in order to do this, however I
think that I will end up creating a more robust setup in the process. At least I
will create a setup that allows me to more easily maintain my own backups rather
than just relying on DigitalOcean snapshots and praying like I do with the
Kubernetes setup.

Thanks farcaller, Marbles, John Rinehart and others for reviewing this post
prior to it being published. 
