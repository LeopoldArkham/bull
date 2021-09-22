# Bull

## What?
Bull is a simple build runner that can watch GitHub repositories

## Why?
Bull was created to make it easy to deploy your project to a VPS environment

## MVP
- Manually register GH webhooks
- Web interface through which I can add a repo/branch pair and a build and a run command to be executed when
a new commit is pushed to it
- On each new commit, run the build and run commands on the updated code with any env vars specified on the web interface
- Display output of build command on web interface
