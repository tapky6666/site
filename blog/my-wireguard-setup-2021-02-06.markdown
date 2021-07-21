---
title: "My Automagic NixOS Wireguard Setup"
date: 2021-02-06
tags:
 - wireguard
 - nixos
 - tailscale
---

It's been a while since I went into detail about how my [Site to Site
Wireguard](/blog/series/site-to-site-wireguard) setup works. I've had a lot of
time to think about how I can improve it since then, and I think I've come to a
new setup that I'm happy with. I've replaced all of the manual setup,
copying/pasting and more with a unified [network metadata
file](https://github.com/Xe/nixos-configs/blob/master/ops/metadata/hosts.toml)
and some generators that consume it. Here's my logic, influences and the details
about how I implemented it.

When I worked at [IMVU](https://secure.imvu.com/) one of the most useful
services was the asset database. This database ended up being a giant bag of
state that a lot of the other SRE services consumed. This was used by the
machine provisioner, DHCP server and the configuration management. My personal
infrastructure isn't quite big enough yet to justify setting up a whole database
for tracking it all, however I think I have a happy middle path with a file
called `hosts.toml`.

At a high level it contains the following information:

- IP subnets that I use across my infrastructure
- Descriptions of the logical subnets they fall into (usually based on physical
  location with a few special exceptions)
- Host information including SSH/wireguard pubkeys

Here's a random host description from `hosts.toml`:

```toml
[hosts.shachi]
network = "hexagone"
ip_addr = "192.168.0.177"
ssh_pubkey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIL3Jt26HXD7mLNjg+B+pB5+fXtxEmMeR6Bqv1Z5/819n"

[hosts.shachi.wireguard]
pubkey = "S8XgS18Z8xiKwed6wu9FE/JEp1a/tFRemSgfUl3JPFw="
port = 51820
addrs = { v4 = "10.77.2.8", v6 = "ed22:a601:31ef:e676:e9bd" }
```

This includes enough information for me to do the following things:

- Set up prometheus monitoring probes
- Send this host [encrypted secrets using its SSH host
  key](/blog/nixos-encrypted-secrets-2021-01-20)
- Configure the Wireguard tunnel that my machines use to talk to eachother

I also have two functions that generate [peer
configs](https://search.nixos.org/options?channel=20.09&from=0&size=50&sort=relevance&query=networking.wireguard.interfaces.%3Cname%3E.peers)
from this metadata,
[roamPeer](https://github.com/Xe/nixos-configs/blob/master/ops/metadata/peers.nix#L5-L17)
and
[serverPeer](https://github.com/Xe/nixos-configs/blob/master/ops/metadata/peers.nix#L18-L33).

The main difference between these two functions is that serverPeer allows me to
tell the target machine to actively reach out to the peer, whereas roamPeer sets
up config for the other end to connect to that peer. This allows me to stick a
machine behind a NAT firewall and still have it connect to the network.

I have two main peerlists based on the location of the machine in question:

```nix
# expected peer lists
hexagone = [
  # cloud
  (serverPeer lufta)
  (serverPeer firgu)
  (serverPeer kahless)
  # hexagone
  (serverPeer chrysalis)
  (serverPeer keanu)
  (serverPeer shachi)
  (serverPeer genza)
];

cloud = [
  # cloud
  (serverPeer lufta)
  (serverPeer firgu)
  (serverPeer kahless)
  # hexagone
  (roamPeer chrysalis)
  (roamPeer keanu)
  (roamPeer shachi)
  (roamPeer genza)
];
```

Inside `hexagone`, all of the machines can freely contact eachother. These IP
addresses aren't very useful for cloud servers, so those servers get a roaming
peer config.

Now that I have these peer lists all I need to do is generate the base Wireguard
config for that machine. At a minimum we need to set the following:

- IP addresses
- The Wireguard private key location
- The Wireguard listen port
- The list of peers

So we do this in the very imaginatively named function `interfaceInfo`:

```nix
interfaceInfo = { network, wireguard, ... }:
  peers:
  let
    net = metadata.networks."${network}";
    v6subnet = net.ula;
  in {
    ips = [
      "${metadata.common.ula}:${wireguard.addrs.v6}/128"
      "${metadata.common.gua}:${wireguard.addrs.v6}/128"
      "${wireguard.addrs.v4}/32"
    ];
    privateKeyFile = "/root/wireguard-keys/private";
    listenPort = wireguard.port;
    inherit peers;
  };
```

`interfaceInfo` takes host information from `hosts.toml` and combines it with a
peerlist in order to tell NixOS all it needs to set up the Wireguard interface.
With this information plus the peerlists from before, we can set up host
configurations:

```nix
hosts = {
  # hexagone
  chrysalis = interfaceInfo chrysalis hexagone;
  keanu = interfaceInfo keanu hexagone;
  shachi = interfaceInfo shachi hexagone;
  genza = interfaceInfo genza hexagone;

  # cloud
  lufta = interfaceInfo lufta cloud;
  firgu = interfaceInfo firgu cloud;
};
```

And then I can set up a `akua.nix` file in the host configuration folder that
looks something like this:

```nix
{ config, pkgs, ... }:

let metadata = pkgs.callPackage ../../ops/metadata/peers.nix { };
in {
  networking.wireguard.interfaces.akua =
    metadata.hosts."${config.networking.hostName}";
    
  within.secrets.wg-privkey = {
    source = ./secrets/wg.privkey;
    dest = "/root/wireguard-keys/private";
    owner = "root";
    group = "root";
    permissions = "0400";
  };
}
```

Then when I push to my machines next, the new Wireguard config will be pushed
across the network, seamlessly integrating any new machine into the mesh.

[Wait, you have other machines like an iPad, iPhone and MacBook and I didn't see
you detail those anywhere in this network. How do you manage Wireguard for
them?](conversation://Mara/hmm)

I don't!

I actually use Tailscale's [subnet
routing](https://tailscale.com/kb/1019/subnets) to handle this. I have my tower
at home expose a route for `10.77.0.0/16` and then it all works out
automagically. Sure it doesn't expose _everything_ if my tower goes and stays
down, however in that case I'm probably going to just make one of my cloud
servers into the subnet router.

<small>Small disclaimer: Tailscale is my employer. I am not speaking for them
with this section. I use them for this because it solves the problem I have
with this so well that I don't have to care about this anymore. Seriously this
has removed so much manual process from my Wireguard networks it's not even
funny. I was a Tailscale user before I was a Tailscale employee.</small>

[Is it really a good idea to include those Wireguard public keys in a public git
repo like that?](conversation://Mara/hmm)

They are _public_ keys, however I have no idea if it really is a good idea or
not. It hasn't gotten me hacked yet (as far as I'm aware), so there's probably
not much of a practical issue.

My logic behind making my NixOS config repo a public one is to act as an example
for others to take inspiration from. I also wanted to make it harder for me to
let the config drift. It also gives me a bunch of fodder for this blog.

---

This is basically what my setup has turned into. It's super easy to manage now.
If I want to add machines to the network, I just generate a new wirguard keypair,
modify `hosts.toml` and then push out the config to the network. That's it. It's
beautiful and I love it.

Feel free to take inspiration from this setup. I'm sure you can do it in a nicer
way somehow (maybe put the metadata table into the nix file itself? that way it
would work on NixOS stable), but this works amazingly for my needs.
