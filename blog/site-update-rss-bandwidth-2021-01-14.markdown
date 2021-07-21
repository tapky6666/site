---
title: "Site Update: RSS Bandwidth Fixes"
date: 2021-01-14
tags:
 - devops
 - optimization
---

Well, so I think I found out where my Kubernetes cluster cost came from. For
context, this blog gets a lot of traffic. Since the last deploy, my blog has
served its RSS feed over 19,000 times. I have some pretty naiive code powering
the RSS feed. It basically looked something like this:

- Write RSS feed content-type and beginning of feed
- For every post I have ever made, include its metadata and content
- Write end of RSS feed

This code was _fantastically simple_ to develop, however it was very expensive
in terms of bandwidth. When you add all this up, my RSS feed used to be more
than a _one megabyte_ response. It was also only getting larger as I posted more
content.

This is unsustainable, so I have taken multiple actions to try and fix this from
several angles.

<blockquote class="twitter-tweet"><p lang="en" dir="ltr">Rationale: this is my
most commonly hit and largest endpoint. I want to try and cut down its size.
<br><br>current feed (everything): 1356706 bytes<br>20 posts: 177931 bytes<br>10
posts: 53004 bytes<br>5 posts: 29318 bytes <a
href="https://t.co/snjnn8RFh8">pic.twitter.com/snjnn8RFh8</a></p>&mdash; Cadey
A. Ratio (@theprincessxena) <a
href="https://twitter.com/theprincessxena/status/1349892662871150594?ref_src=twsrc%5Etfw">January
15, 2021</a></blockquote> <script async
src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>

[Yes, that graph is showing in _gigabytes_. We're so lucky that bandwidth is
free on Hetzner.](conversation://Mara/hacker)

First I finally set up the site to run behind Cloudflare. The Cloudflare
settings are set very permissively, so your RSS feed reading bots or whatever
should NOT be affected by this change. If you run into any side effects as a
result of this change, [contact me](/contact) and I can fix it.

Second, I also now set cache control headers on every response. By default the
"static" pages are cached for a day and the "dynamic" pages are cached for 5
minutes. This should allow new posts to show up quickly as they have previously.

Thirdly, I set up
[ETags](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/ETag) for the
feeds. Each of my feeds will send an ETag in a response header. Please use this
tag in future requests to ensure that you don't ask for content you already
have. From what I recall most RSS readers should already support this, however
I'll monitor the situation as reality demands.

Lastly, I adjusted the
[ttl](https://cyber.harvard.edu/rss/rss.html#ltttlgtSubelementOfLtchannelgt) of
the RSS feed so that compliant feed readers should only check once per day. I've
seen some feed readers request the feed up to every 5 minutes, which is very
excessive. Hopefully this setting will gently nudge them into behaving.

As a nice side effect I should have slightly lower ram usage on the blog server
too! Right now it's sitting at about 58 and a half MB of ram, however with fewer
copies of my posts sitting in memory this should fall by a significant amount.

If you have any feedback about this, please [contact me](/contact) or mention me
on Twitter. I read my email frequently and am notified about Twitter mentions
very quickly.
