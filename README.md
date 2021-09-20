# Bull

## What?
Bull is a simple build runner that can watch GitHub repositories

## Why?
Bull was created to make it easy to deploy your project to a VPS environment

## Assumptions
- The GitHub hooks will be registered manually
- There are no ENV vars neded for the compilation
- The toolchains necessary to runthe build command are already installed
- Downtime between the teardown of the old version and running the new version is permissible
- Repos are public