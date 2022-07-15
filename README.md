# autobean
This is a program for verifying the correctness of my own Beancount accounts.

# Usage
The linter can be executed with docker, just mount the directory with your .beancount files into the `/data` directory as below:
```shell
$ docker run -it --rm -v "/path/to/data-directory:/data" ghcr.io/datavirke/autobean:main /data
```
The program will print any warnings or issues to stderr.

## As part of a GitHub workflow
See the [github workflow example](/examples/github-workflow.yml)

# Building
Building uses Docker BuildKit to speed up compilation by caching dependencies and build artifacts, so either enable it in your [Docker daemon configuration](https://docs.docker.com/develop/develop-images/build_enhancements/) or preface the below command with `DOCKER_BUILDKIT=1` to enable it for the build.
```shell
$ docker build -t ghcr.io/datavirke/autobean:test .
```

# Notes
Some of the lints make assumptions about the way beancount is being used.

For example, the appendix checks ensure the following:
* All transactions must have a `statement` clause.
* Any `statement` clause is a link to an appendix used for auditing purposes, typically a pdf containing invoice or receipt.
* Many transactions can refer to the same appendix.
* The format of the `statement` clause is `any/prefix-path/YYYY-MM-DD.<Appendix ID>.*` where the `YYYY-MM-DD` format is a date, and the Appendix ID:
    * Is an unsigned integer starting with 1 and incrementing.
    * Is unique: that is no two appendices may have the same ID.
    * Are sequential. There can be no gaps in the IDs across the entire ledger.