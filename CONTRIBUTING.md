# Contributing

Thank you for your interest in contributing to tendermint-rs! The goal of
tendermint-rs is to provide a high quality, formally verified implementation of
Tendermint.

All work on the code base should be motivated by a Github Issue. Search is a
good place start when looking for places to contribute. If you would like to
work on an issue which already exists, please *assign yourself* to the issue if
nobody else is already assigned to it. If someone else is already assigned to an
issue, leave a comment on the issue to indicate that you'd like to work on it,
or contact the assignee directly.

If you'd like to work on something else, open an Issue to start the discussion.

The rest of this document outlines the best practices for contributing to this
repository:

- [Decision Making](#decision-making) - process for agreeing to changes
- [Forking](#forking) - fork the repo to make pull requests
- [Changelog](#changelog) - changes must be recorded in the changelog
- [Pull Requests](#pull-requests) - what makes a good pull request
- [Releases](#releases) - how our release process looks

## Decision Making

When contributing to the project, the following process leads to the best chance
of landing the changes in master.

All new contributions should start with a Github Issue. The issue helps capture
the problem you're trying to solve and allows for early feedback. Once the issue
is created, maintainers may request more detailed documentation be written in
the form of a Request for Comment (RFC) or Architectural Decision Record
([ADR](https://github.com/informalsystems/tendermint-rs/blob/master/docs/architecture/README.md)).

Discussion at the RFC stage will build collective understanding of the
dimensions of the problems and help structure conversations around trade-offs.

When the problem is well understood but the solution leads to large structural
changes to the code base, these changes should be proposed in the form of an
[Architectural Decision Record (ADR)](./docs/architecture/). The ADR will help
build consensus on an overall strategy to ensure the code base maintains
coherence in the larger context. If you are not comfortable with writing an ADR,
you can open a less-formal issue and the maintainers will help you turn it into
an ADR.

When the problem as well as proposed solution are well understood, changes
should start with a [draft pull
request](https://github.blog/2019-02-14-introducing-draft-pull-requests/)
against master. The draft signals that work is underway. When the work is ready
for feedback, hitting "Ready for Review" will signal to the maintainers to take
a look.

Implementation trajectories should aim to proceed where possible as a series of
smaller incremental changes, in the form of small PRs that can be merged
quickly. This helps manage the load for reviewers and reduces the likelihood
that PRs will sit open for longer.

![Contributing
flow](https://github.com/tendermint/tendermint/blob/v0.33.6/docs/imgs/contributing.png)

Each stage of the process is aimed at creating feedback cycles which align
contributors and maintainers to make sure:

- Contributors don’t waste their time implementing/proposing features which
  won’t land in `master`.
- Maintainers have the necessary context in order to support and review
  contributions.

## Forking

If you do not have write access to the repository, your contribution should be
made through a fork on Github. Fork the repository, contribute to your fork, and
make a pull request back upstream.

When forking, add your fork's URL as a new git remote in your local copy of the
repo. For instance, to create a fork and work on a branch of it:

- Create the fork on GitHub, using the fork button.
- `cd` to the original clone of the repo on your machine
- `git remote rename origin upstream`
- `git remote add origin git@github.com:<location of fork>

Now `origin` refers to your fork and `upstream` refers to this version.

`git push -u origin master` to update the fork, and make pull requests against
this repo.

To pull in updates from the origin repo, run

- `git fetch upstream`
- `git rebase upstream/master` (or whatever branch you want)

## Changelog

Every non-trivial PR must update the [CHANGELOG.md]. This is accomplished
indirectly by adding entries to the `.changelog` folder in [unclog][unclog]
format. `CHANGELOG.md` will be built by whomever is responsible for performing a
release just prior to release - this is to avoid changelog conflicts prior to
releases.

The Changelog is *not* a record of which pull requests were merged; the commit
history already shows that. The Changelog is a notice to the user about how
their expectations of the software should be modified.  It is part of the UX of
a release and is a *critical* user facing integration point.  The Changelog must
be clean, inviting, and readable, with concise, meaningful entries.  Entries
must be semantically meaningful to users. If a change takes multiple Pull
Requests to complete, it should likely have only a single entry in the Changelog
describing the net effect to the user.

When writing Changelog entries, ensure they are targeting users of the software,
not fellow developers. Developers have much more context and care about more
things than users do. Changelogs are for users.

Changelog structure is modeled after [Tendermint
Core](https://github.com/tendermint/tendermint/blob/master/CHANGELOG.md) and
[Hashicorp Consul](http://github.com/hashicorp/consul/tree/master/CHANGELOG.md).
See those changelogs for examples.

Changes for a given release should be split between the five sections: Security,
Breaking Changes, Features, Improvements, Bug Fixes.

Changelog entries should be formatted as follows:

```
- `[pkg]` A description of the change with *users* in mind
  ([#xxx](https://github.com/informalsystems/tendermint-rs/issues/xxx))
```

Here, `pkg` is the part of the code that changed, and `xxx` is the issue or
pull-request number.

Changelog entries should be ordered alphabetically according to the `pkg`, and
numerically according to the issue/pull-request number.

Changes with multiple classifications should be doubly included (eg. a bug fix
that is also a breaking change should be recorded under both).

Breaking changes are further subdivided according to the APIs/users they impact.
Any change that effects multiple APIs/users should be recorded multiply - for
instance, a change to some core protocol data structure might need to be
reflected both as breaking the core protocol but also breaking any APIs where
core data structures are exposed.

## Pull Requests

The master development branch is `master`. 
Branch names should be prefixed with the author, eg. `name/feature-x`. 

Pull requests are made against `master`
and are squash merged into master.

PRs must:

- make reference to an issue outlining the context.
- update any relevant documentation and include tests.
- update the [changelog](#changelog) with a description of the change

Pull requests should aim to be small and self contained to facilitate quick
review and merging. Larger change sets should be broken up across multiple PRs.
Commits should be concise but informative, and moderately clean. Commits will be squashed into a
single commit for the PR with all the commit messages.

## Releases

Our release process is as follows:

1. Update the [changelog](#changelog) to reflect and summarize all changes in
   the release. This involves:
   1. Running `unclog release vX.Y.Z` to create a summary of all of the changes
      in this release.
   2. Running `unclog build > CHANGELOG.md` to update the changelog.
   3. Committing this updated `CHANGELOG.md` file to the repo.
2. Push this to a branch `release/vX.Y.Z` according to the version number of
   the anticipated release (e.g. `release/v0.17.0`) and open a **draft PR**.
3. Bump all relevant versions in the codebase to the new version and push these
   changes to the release PR. This includes:
   1. All `Cargo.toml` files (making sure dependencies' versions are updated
      too).
   2. All crates' `lib.rs` files documentation references' `html_root_url`
      parameters must point to the new version.
4. Run `cargo doc --all-features --open` locally to double-check that all the
   documentation compiles and seems up-to-date and coherent. Fix any potential
   issues here and push them to the release PR.
5. Mark the PR as **Ready for Review** and incorporate feedback on the release.
6. Once approved, run the [`release.sh`] script. Fix any problems that may
   arise during this process and push the changes to the release PR.
   This step requires the appropriate privileges to push crates to [crates.io].
7. Once all crates have been successfully released, merge the PR to `master`
   and tag the repo at the new version (e.g. `v0.17.0`).

[CHANGELOG.md]: https://github.com/informalsystems/tendermint-rs/blob/master/CHANGELOG.md
[`release.sh`]: https://github.com/informalsystems/tendermint-rs/blob/master/release.sh
[crates.io]: https://crates.io
[unclog]: https://github.com/informalsystems/unclog
