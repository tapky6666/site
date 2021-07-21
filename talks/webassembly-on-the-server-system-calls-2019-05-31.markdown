---
title: "WebAssembly on the Server: How System Calls Work"
date: 2019-05-31
slides_link: /static/talks/wasm-on-the-server-system-calls.pdf
---

# WebAssembly on the Server: How System Calls Work

[Video](https://www.youtube.com/watch?v=G4l8RX0tA3E)

## My Speaker Notes

* Hi, my name is Christine. I work as a senior SRE for Lightspeed. Today I'm gonna talk about something I've been researching and learning a lot about: WebAssembly on the server.
* Something a lot of you might be asking: what is WebAssembly?
    * WebAssembly is very new and there's a lot of confusing and overly vague coverage on it. 
    * In this talk, I will explain WebAssembly at a high level and show how to start solving one of the hardest problems in it: how to communicate with the outside world.
    * When I say the "outside world" I mean anything that is not literally one of these 5 basic things:
        * Externally imported functions, defined by the user
        * The dynamic dispatch function table
        * Global variables
        * Linear memory, or basically ram
        * Compiled functions, or your code that runs in the virtual machine
* WebAssembly is a Virtual Machine format for the Web
    * The closest analogue to WASM in its current form is a CPU and supporting hardware
    * However, because it's a virtual machine, the hardware is irrelevant
    * Though it was intended for browsers, the implementation of it is really generic.
    * WebAssembly provides:
        * External functions
        * A function table for dynamic dispatch
        * Immutable globals (as of the MVP)
        * Linear memory
        * Compiled functions (these exist outside of linear memory like an AVR chip)
* Why WebAssembly on the Server?
    * It makes hardware less relevant.
    * Most of our industry targets a single vendor in basic configurations: Intel amd64 processors running Linux
        * Intel has had many security bugs and it may not be a good idea to fundamentally design our architecture to rely on them.
    * This also removes the OS from the equation for most compute tasks.
* What are system calls and why do they matter?
* System calls enforce abstractions to the outside world.
    * Your code goes through system calls to reach things from the outside world, eg:
        * Randomness
        * Network sockets
        * The filesystem
        * Etc
* How are they implemented?
    * The platform your program runs on exposes those system calls
    * Programs pass pointers into linear memory (this will be shown later in the slides)
* Why is this relevant to WebAssembly?
    * The WebAssembly Minimum Viable Product doesn't define any system calls
* WebAssembly System Calls Out of The Box
    * Yeah, nothing. You're on your own. This is both very good and very very bad.
* So what's a pointer in WebAssembly?
    * Simplified, a WebAssembly virtual machine is some structure that has a reference to a byte slice. That byte slice is treated as the linear memory of that VM.
    * A pointer is just an offset into this slice
    * Showing the WebAssembly world diagram from earlier: pointers apply to only this part of it. Function pointers _do_ exist in WebAssembly, just by the dynamic dispatch table from earlier.
* So what can we do about it?
* Let's introduce a pet project of mine for a few years. It's called Dagger, and it has been a fantastic stepping stone while other solutions are being invented.
    * Dagger is a proof of concept system call API that I'll be walking through the high level implementation of
    * It's got a very simple implementation (500-ish lines)
    * It's intended for teaching and learning about the low levels of WebAssembly.
    * It's based on a very very very simplistic view of the unix philosophy. In unix, everything is a file. With Dagger, everything is a stream, even HTTP.
    * As such, there's no magic in Dagger.
    * And even though it's so simple, it's still usable for more than just basic/trivial things.
    * A dagger process has a bunch of streams in a slice.
    * The API gives out and uses stream descriptors, or offsets into this slice.
* Dagger's API is really really simple, it's only got 5 calls:
    * Opening a stream
    * Closing a stream
    * Reading from a stream
    * Writing to a stream
    * Flushing intermediately buffered data from a stream to its remote (or local) target
* Open
    * Open opens a stream by URL, then returns its descriptor. It can also return an error instead.
    * It's got 5 basic stream types:
        * Logging
        * Jailed filesystem access
        * HTTP/S
            * 5 system calls is all you need for HTTP!
        * Randomness
        * Standard input/output
    * Let's walk through the code that implements it
        * Here's a simplified view of the open function in a Dagger process.
        * The system call arguments are here
        * And the stream URL gets read from the VM memory here
        * Remember that pointers are just integer offsets into memory
        * Then this gets passed to the rest of the open file logic that isn't shown here
* Close
    * Closes a stream by its descriptor.
    * It returns a negative error if anything goes wrong, which is unlikely.
    * Let's walk through its code:
        * It grabs the arguments from the VM
        * Then it passes that to the rest of the logic that isn't shown here
* Read
    * Reads a limited amount of bytes from a stream
    * Returns a negative error if things go wrong
    * Let's walk through its code:
        * This is a bigger function, so I've broken it up into a few slides.
        * First it gets the arguments from the VM
        * Then it creates the intermediate buffer to copy things into from the stream
        * Then it does the reading into that buffer
        * Then it copies the buffer into the VM ram
* Write
    * Write is very similar to read, except it just copies the ram out of the VM and into the stream
    * It returns the number of bytes written, which SHOULD equal the data length argument
    * Let's walk through the code:
        * Again, this function is bigger so I 
* Flush
    * Flush does just about what you'd think, it flushes intermediate buffers to the actual stream targets.
    * This blocks until the flushing is complete
    * Mostly used for the HTTP client
    * Let's walk through its code:
        * It gets the descriptor from the VM
        * It runs the flush operation and returns the result
* So, with all this covered, let's talk about usage. Here's the famous "Hello, world" example:
    * This is in Zig, mainly because Zig allows me to be really concise. Things work just about as you'd expect so it's less of a logical jump than you'd think.
    * First we try to open the stream. Dagger doesn't have any streams open in its environment by default, so we open standard output.
    * Then we try to write the message to the stream. The interface in Zig is a bit rough right now, but it takes the pointer to the message and how long the message is. Zig doesn't let us implicitly ignore the return value of this function, so we just explicitly ignore it instead.
    * Finally we try to close the output stream.
    * The beauty of zig is that if any of these things we try to do fails, the entire function will fail.
    * However none of this fails so we can just run it with the dagger tool and get this output:
* What this can build to
    * This basic idea can be used to build up to any of the following things:
        * A functions as a service backend (See Olin)
        * Generic event handlers
        * Distributed computing
        * Transactional computing
* What you can do
    * Play with the code (link at the end)
    * Implement this API from scratch
        * It's really not that hard
    * A possible project idea I was going to do but ran out of time (moving internationally sucks) is to make a Gopher server with every route powered by WebAssembly
* Got questions?
    * Tweet or email me if you really want to make sure your questions get answered. That is one of the best ways to ensure I actually see it.
    * I'm happy to go into detail, I can pull out code examples too.
* Thanks to all of these people who have given help, ideas and inspiration. Without them I would never have been able to get this far.
* Follow my progress on GitHub!
    * I hope that QR code is big enough. If it's not let me know and I can make things like that bigger in the future somehow, hopefully.
