# Contributing to this project

Thank you for considering contributing to this project!
I welcome bug reports, feature requests, and pull requests. Please follow the guidelines below to help us maintain a clean and consistent codebase.

## Development branch policy

All contributions should be based on the `dev` branch. Do **not** open pull requests directly against `main`.

1. Fork the repository.
2. Create a new branch from `dev`, e.g. `feature/add-awesome-flag`.
3. Submit your pull request against the `dev` branch.
4. Once approved, it will be merged into `dev`, and then `dev` will be merged into `main` via a version branch by a maintainer.

## Pull Request Guidelines

- Keep pull requests focused and limited to a single purpose.
- Use clear and descriptive titles for your PRs.
- Run `cargo fmt` and `cargo clippy` before submitting.

## Merge Strategy

We use **Squash and Merge** as the default strategy, unless otherwise justified. This ensures a clean and readable commit history.

## Licensing

By submitting your contribution, you agree that your code will be licensed under MIT license as the project.
