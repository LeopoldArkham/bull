# Bull

## What?
Bull is a simple build runner that can watch GitHub repositories

## Why?
Bull was created to make it easy to deploy your projects to a VPS you own

## MVP
- Manually register GH webhooks
- Web interface through which I can add a repo/branch pair and a build and a run command to be executed when
a new commit is pushed to it
- On each new commit, run the build and run commands on the updated code with any env vars specified on the web interface
- Display output of build command on web interface


## Running locally
Use Ngrok to provide a tunnel to localhost. After installing, run this command:
```
ngrok http -host-header=rewrite localhost:<port>
```

where `<port>` is the port that you have configured Bull to listen on. Use the https forwarding address provided by ngrok as the payload URL for webhooks in the repo you want to watch.
Note that using the free version of ngrok, that address will expire every two hours, at which point the command will have to be run again, and the payload address updated if it has changed