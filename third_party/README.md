This directory contains synchronized copies of external repositories (currently only Move). Code can be submitted in this directory using a single atomic PR. Periodically, changes in this directory
are pushed upstream or pulled from upstream, using the [copybara](https://github.com/google/copybara) tool.

## Usage

Assuming `copybara` is available from the command line, to pull from the
Move repo (for example), use:

### Pulling

```shell
copybara copy.bar.sky pull_move
```

This will create a draft PR in aptos_core (in the fixed branch `from_move`) with the needed changes. The PR should be massaged and send out for regular review.

### Pushing

In order to push to the Move repo, use:

```shell
copybara copy.bar.sky push_move
```

This will directly push to the `aptos_main` branch in the Move repo.


## Installing Copybara

Copybara must be build from source. 

### MacOS

We first need Java. If its not yet in your path (`java` should show), you can install the openjdk with relative little hassle:

```shell
brew update
brew install java
```

The last step should print out instructions how to update the PATH so `java` is found.

We also need bazel:

```shell
brew install bazel
```

Finally we can clone the copybara repo and compile the program:

```shell
git clone https://github.com/google/copybara.git
cd copybara
bazel build //java/com/google/copybara
alias copybara="$PWD/bazel-bin/java/com/google/copybara/copybara"
```

### Linux

TBD