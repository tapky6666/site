---
title: Using Morph for Deploying to NixOS
date: 2021-04-25
series: nixos
tags:
 - morph
---

Managing a single NixOS host is easy. Any time you want to edit any settings,
you can just change options in `/etc/nixos/configuration.nix` and then do
whatever you want from there. Managing multiple NixOS machines can be
complicated. [Morph](https://github.com/DBCDK/morph) is a tool that makes it
easy to manage multiple NixOS machines as if they were one single machine. In
this post we're gonna start a new NixOS configuration for a network of servers
from scratch and explain each step in the way.

## `nixos-configs` Repo

NixOS configs usually need a home. We can make a home for this in a Git
repository named `nixos-configs`. You can make a nixos configs repo like this:

```console
$ mkdir -p ~/code/nixos-configs
$ cd ~/code/nixos-configs
$ git init
```

[You can see a copy of the repo that we're describing in this post <a
href="https://github.com/Xe/blog-nixos-configs">here</a>. That repo is licensed
as Creative Commons Zero and no attribution or credit is required if you want to
use it as the basis for your NixOS configuration repo for any setup, home or
professional.](conversation://Mara/hacker)

From here you could associate it with a Git forge if you want, but that is an
exercise left to the reader.

Now that we have the nixos-configs repository, create a few folders that
will be used to help organize things:

- `common` -> base system configuration and options
- `common/users` -> user account configuration
- `hosts` -> host-specific configuration for named servers
- `ops` -> operations data such as deployment configuration
- `ops/home` -> configuration for a home network

You can make them with a command like this:

```console
$ mkdir -p common/users hosts ops/home
```

Now that we have the base layout, start with adding a few files into the
`common` folder:

- `common/default.nix` -> the "parent" file that will import all of the other
  files in the `common` directory, as well as define basic settings that
  everything else will inherit from
- `common/generic-libvirtd.nix` -> a bunch of settings to configure libvirtd
  virtual machines (omit this if you aren't running VMs in libvirtd)
- `common/users/default.nix` -> the list of all the user accounts we are going
  to configure in this system
  
Here's what you should put in `common/default.nix`:

```nix
# common/default.nix

# Mara\ inputs to this NixOS module. We don't use any here
# so we can ignore them all.
{ ... }:

{
  imports = [
    # Mara\ User account definitions
    ./users
  ];
 
  # Mara\ Clean /tmp on boot.
  boot.cleanTmpDir = true;
  
  # Mara\ Automatically optimize the Nix store to save space
  # by hard-linking identical files together. These savings
  # add up.
  nix.autoOptimiseStore = true;
  
  # Mara\ Limit the systemd journal to 100 MB of disk or the
  # last 7 days of logs, whichever happens first.
  services.journald.extraConfig = ''
    SystemMaxUse=100M
    MaxFileSec=7day
  '';

  # Mara\ Use systemd-resolved for DNS lookups, but disable
  # its dnssec support because it is kinda broken in
  # surprising ways.
  services.resolved = {
    enable = true;
    dnssec = "false";
  };
}
```

This will give you a base system config with sensible defaults that you can
build on top of.

[Is now when I get my account? :D](conversation://Mara/happy)

[Yep! We define that in `common/users/default.nix`:](conversation://Cadey/enby)

```nix
# common/users/default.nix

# Mara\ Inputs to this NixOS module, in this case we are
# using `pkgs` so I can configure my favorite shell fish
# and `config` so we can make my SSH key also work with
# the root user.
{ config, pkgs, ... }:

{
  # Mara\ The block that specifies my user account.
  users.users.mara = {
    # Mara\ This account is intended for a non-system user.
    isNormalUser = true;
    
    # Mara\ The shell that the user will default to. This
    # can be any NixOS package, even PowerShell!
    shell = pkgs.fish;
    
    # Mara\ My SSH keys.
    openssh.authorizedKeys.keys = [
      # Mara\ Replace this with your SSH key!
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIPg9gYKVglnO2HQodSJt4z4mNrUSUiyJQ7b+J798bwD9"
    ];
  };
  
  # Mara\ Use my SSH keys for logging in as root.
  users.users.root.openssh.authorizedKeys.keys =
    config.users.users.mara.openssh.authorizedKeys.keys;
}
```

In case you are using libvirtd to test this blogpost like I am, put the
following in `common/generic-libvirtd.nix`:

```nix
# common/generic-libvirtd.nix

# Mara\ This time all we need is the `modulesPath`
# to grab an optional module out of the default
# set of modules that ships in nixpkgs.
{ modulesPath, ... }:

{
  # Mara\ Set a bunch of QEMU-specific options that
  # aren't set by default.
  imports = [ (modulesPath + "/profiles/qemu-guest.nix") ];

  # Mara\ Enable SSH daemon support.
  services.openssh.enable = true;

  # Mara\ Make sure the virtual machine can boot
  # and attach to its disk.
  boot.initrd.availableKernelModules =
    [ "ata_piix" "uhci_hcd" "virtio_pci" "sr_mod" "virtio_blk" ];

  # Mara\ Other boot settings that we're leaving
  # to the defaults.
  boot.initrd.kernelModules = [ ];
  boot.kernelModules = [ ];
  boot.extraModulePackages = [ ];

  # Mara\ This VM boots with grub.
  boot.loader.grub.enable = true;
  boot.loader.grub.version = 2;
  boot.loader.grub.device = "/dev/vda";

  # Mara\ Mount /dev/vda1 as the root filesystem.
  fileSystems."/" = {
    device = "/dev/vda1";
    fsType = "ext4";
  };
}
```

Now that we have the basic modules defined, we can create a `network.nix` file
that will tell Morph where to deploy to. In this case we are going to create a
network with a single host called `ryuko`. Put the following in
`ops/home/network.nix`:

```nix
# ops/home/network.nix

{
  # Mara\ Configuration for the network in general.
  network = {
    # Mara\ A human-readable description.
    description = "My awesome home network";
  };

  # Mara\ This specifies the configuration for
  # `ryuko` as a NixOS module.
  "ryuko" = { config, pkgs, lib, ... }: {
    # Mara\ Import the VM-specific config as
    # well as all of the settings in
    # `common/default.nix`, including my user
    # details.
    imports = [
      ../../common/generic-libvirtd.nix
      ../../common
    ];
    
    # Mara\ The user you will SSH into the
    # machine as. This defaults to your current
    # username, however for this example we will
    # just SSH in as root.
    deployment.targetUser = "root";
    
    # Mara\ The target IP address or hostname
    # of the server we are deploying to. This is
    # the IP address of a libvirtd virtual
    # machine on my machine.
    deployment.targetHost = "192.168.122.251";
  };
}
```

Now that we finally have all of this set up, we can write a little script that
will push this config to the server by doing the following:

- Build the NixOS configuration for `ryuko`
- Push the NixOS configuration for `ryuko` to the virtual machine
- Activate the configuration on `ryuko`

Put the following in `ops/home/push`:

```shell
#!/usr/bin/env nix-shell
# Mara\ The above shebang line will use `nix-shell`
# to create the environment of this shell script.

# Mara\ Specify the packages we are using in this
# script as well as the fact that we are running it
# in bash.
#! nix-shell -p morph -i bash

# Mara\ Explode on any error.
set -e

# Mara\ Build the system configurations for every
# machine in this network and register them as
# garbage collector roots so `nix-collect-garbage`
# doesn't sweep them away.
morph build --keep-result ./network.nix

# Mara\ Push the config to the hosts.
morph push ./network.nix

# Mara\ Activate the NixOS configuration on the
# network.
morph deploy ./network.nix switch
```

Now mark that script as executable:

```console
$ cd ./ops/home
$ chmod +x ./push
```

And then try it out:

```console
$ ./push
```

And finally SSH into the machine to be sure that everything works:

```console
$ ssh mara@192.168.122.251 -- id
uid=1000(mara) gid=100(users) groups=100(users)
```

From here you can do just about anything you want with `ryuko`.

If you want to add a non-VM NixOS host to this, make a folder in `hosts` for
that machine's hostname and then copy the contents of `/etc/nixos` to that
folder. For example if you have a server named `mako` with the IP address
`192.168.122.147`. You would do something like this:

```console
$ mkdir hosts/mako -p
$ scp root@192.168.122.147:/etc/nixos/configuration.nix ./hosts/mako
$ scp root@192.168.122.147:/etc/nixos/hardware-configuration.nix ./hosts/mako
```

And then you can register it in your `network.nix` like this:

```nix
"mako" = { config, pkgs, lib, ... }: {
  deployment.targetUser = "root";
  deployment.targetHost = "192.168.122.147";
  
  # Mara\ Import mako's configuration.nix
  imports = [ ../../hosts/mako/configuration.nix ];
};
```

This should help you get your servers wrangled into a somewhat consistent state.
From here the following articles may be useful to give you ideas:

- [Borg Backup Config](https://christine.website/blog/borg-backup-2021-01-09)
- [Nixops Services On Your Home
  Network](https://christine.website/blog/nixops-services-2020-11-09) (just be
  sure to ignore the part where it mentions `deployment.keys`, you can replace
  it with the semantically identical
  [`deployment.secrets`](https://github.com/DBCDK/morph/blob/master/examples/secrets.nix)
  as described in the morph documentation)
- [Prometheus and
  Aegis](https://christine.website/blog/aegis-prometheus-2021-04-05)
- [My Automagic NixOS Wireguard
  Setup](https://christine.website/blog/my-wireguard-setup-2021-02-06)
- [Encrypted Secrets with
  NixOS](https://christine.website/blog/nixos-encrypted-secrets-2021-01-20)

Also feel free to dig around [the `common` folder of my `nixos-configs`
repo](https://github.com/Xe/nixos-configs/tree/master/common). There's a bunch
of examples of things in there that I haven't gotten around to documenting in
this blog yet. Another useful thing you may want to look into is
[home-manager](https://github.com/nix-community/home-manager), which is a tool
that lets you manage your dotfiles across machines. With home-manager I'm able
to set up all of my configurations for everything on a new machine in less than
30 minutes (starting from a blank NixOS server).
