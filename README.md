# DragoDNS

Drago DNS is a dynamic DNS system built for hobbyists, self-hosters, students, or anyone looking to get into networking, self-hosting, or computer systems who dont have the money for or access to a static IP from their ISP.

Drago is named after the Lightning L-Drago 100HF from Beyblade Metal Masters, a game that went unreasonably hard at the Boys and Girls Club when I grew up.

## Drago is made of 4 parts:

1. The front end: this is where users make their account, add their DNS provider access keys, add records they want to have managed by Drago, and make API keys for the Drago Client
2. Drago Client: This is a light weight client built in rust that checks your IP and sends that data to the Drago API for processing
3. Drago API: Manages syncing and propagating of client IPs to DNS records for the desired machine, to limit the requests sent to the DNS provider, we manage the Drago DB
4. Drago DB: Built on planetscale, the Drago DB holds user data, DNS provider access tokens, DNS Zones, DNS Records and Drago Client API keys. The data is much more ephemeral as we sync the IP values every 5 minutes.

> This should not be used for a production level product. This is for individuals to self host their software on hardware at home or in a classroom.
> Since we sync every 5 minutes, you will experience much more downtime than if you just hosted the application on a VPS with a static IP :)
> Note: this was also built by a novice Rustacean so its probably the worst Rust code you've ever seen.

## Drago is built with:

- SvelteKit
- Rust
- MySQL
- A VPS
- Love ❤️
