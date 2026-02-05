#!/bin/bash

if (( $# ==  4 )); then
    USER=$1
    REPO=$2
    REG_TOKEN=$3
elif (( $# == 3 )); then
    USER=$1
    REPO=$2
    ACCESS_TOKEN=$3
    REG_TOKEN=$(curl -sX POST -H "Authorization: token ${ACCESS_TOKEN}" "https://api.github.com/repos/${USER}/${REPO}/actions/runners/registration-token" | jq .token --raw-output)
elif [[ $# != 3 ]] ; then
    echo "not enough arguments: $#. Should be startup.sh <USER> <REPO> <ACCESS_TOKEN>"
    exit 0
fi

./config.sh --unattended --url "https://github.com/${USER}/${REPO}" --token "${REG_TOKEN}" --name "docker-runner-$(tr -dc A-Za-z0-9 </dev/urandom | head -c 13; echo '')"

cleanup() {
    echo "Removing runner..."
    ./config.sh remove --token ${REG_TOKEN}
}

if (( $# ==  4 )); then
    trap 'cleanup; exit 130' EXIT
fi

nice -n10 ./run.sh &
wait $!

# config.sh help output:
#
#
# Commands:
#  ./config.sh         Configures the runner
#  ./config.sh remove  Unconfigures the runner
#  ./run.sh            Runs the runner interactively. Does not require any options.

# Options:
#  --help     Prints the help for each command
#  --version  Prints the runner version
#  --commit   Prints the runner commit
#  --check    Check the runner's network connectivity with GitHub server

# Config Options:
#  --unattended           Disable interactive prompts for missing arguments. Defaults will be used for missing options
#  --url string           Repository to add the runner to. Required if unattended
#  --token string         Registration token. Required if unattended
#  --name string          Name of the runner to configure (default ee091dca5a34)
#  --runnergroup string   Name of the runner group to add this runner to (defaults to the default runner group)
#  --labels string        Custom labels that will be added to the runner. This option is mandatory if --no-default-labels is used.
#  --no-default-labels    Disables adding the default labels: 'self-hosted,Linux,X64'
#  --local                Removes the runner config files from your local machine. Used as an option to the remove command
#  --work string          Relative runner work directory (default _work)
#  --replace              Replace any existing runner with the same name (default false)
#  --pat                  GitHub personal access token with repo scope. Used for checking network connectivity when executing `./run.sh --check`
#  --disableupdate        Disable self-hosted runner automatic update to the latest released version`
#  --ephemeral            Configure the runner to only take one job and then let the service un-configure the runner after the job finishes (default false)

# Examples:
#  Check GitHub server network connectivity:
#   ./run.sh --check --url <url> --pat <pat>
#  Configure a runner non-interactively:
#   ./config.sh --unattended --url <url> --token <token>
#  Configure a runner non-interactively, replacing any existing runner with the same name:
#   ./config.sh --unattended --url <url> --token <token> --replace [--name <name>]
#  Configure a runner non-interactively with three extra labels:
#   ./config.sh --unattended --url <url> --token <token> --labels L1,L2,L3
