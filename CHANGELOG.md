# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/jdrouet/another-html-builder/compare/v0.1.3...v0.2.0) - 2024-12-09

### Added

- allow to use a `std::fmt::Write` and `std::io::Write` as buffer (#7)

### Other

- add badges to readme
- update coverage configuration
- remove deprecated functions
- update readme

## [0.1.3](https://github.com/jdrouet/another-html-builder/compare/v0.1.2...v0.1.3) - 2024-11-30

### Fixed

- add back removed function with deprecation notice
- ensure mutants are handled

### Other

- make sure we don't introduce mutating bugs
- optimize escaping functions

## [0.1.2](https://github.com/jdrouet/another-html-builder/compare/v0.1.1...v0.1.2) - 2024-11-19

### Added

- add functions to add elements depending on optional ([#5](https://github.com/jdrouet/another-html-builder/pull/5))
- add way to write optional attributes ([#4](https://github.com/jdrouet/another-html-builder/pull/4))

### Fixed

- add double quote wrapper around value in Attribute element

### Other

- ensure basic types attributes are valid
- add documentation and code examples
- add example in readme

## [0.1.1](https://github.com/jdrouet/another-html-builder/compare/v0.1.0...v0.1.1) - 2024-11-13

### Fixed

- only escape double quotes in attributes

### Other

- release v0.1.0 ([#1](https://github.com/jdrouet/another-html-builder/pull/1))

## [0.1.0](https://github.com/jdrouet/another-html-builder/releases/tag/v0.1.0) - 2024-11-11

### Added

- update cargo file
- create proof of concept

### Fixed

- update license field
- remove use of nigthly feature
- apply clippy suggestions

### Other

- add workflow for releasing
- add security audit job
- add simple description to readme
- add github action configuration
- add readme
