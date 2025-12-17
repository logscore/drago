# Drago Client

This is where the Drago client goes. It'll run locally on the users machine that they want to host the software on, it will hold an api key and basic config, it will query the DragoAPI every 5 minutes (or maybe longer idk)

Goals:

- Make it small
- Make it simple
- Make it secure

Data sent:
  API key
  IP address
  TImestamp of when sent

## The flow

> This should be a tui tbh

Run the install script
1. Runs the drago init command
2. Init command checks if the user is signed in by checking, if not, 
3. Prompt for the cloudflare api token. Secure storage is optional
4. Prompts the user for the name of the record, the ip, ttl, and proxy status
5. creates an api key, stores it, then deploys the daemon

What does work:

The cli makes the requests to better-auth api and gets the access token back and saves it

What doesnt work:

The api doesnt handle that token at all, so we cant make requests to the api with it from the cli

Because of the above, we cant add api keys, delete api key,s or anything like that.