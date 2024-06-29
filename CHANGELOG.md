## [2.0.0](https://github.com/mobusoperandi/eelco/compare/v1.3.0...v2.0.0) (2024-06-29)

### ⚠ BREAKING CHANGES

* **flake:** nix provided via NIX_CMD_PATH

### Features

* do not rely on nix experimental ([15e84f7](https://github.com/mobusoperandi/eelco/commit/15e84f7845ecdbacb91829d91cf651c2352af439))
* does not rely on a pty ([e8adac0](https://github.com/mobusoperandi/eelco/commit/e8adac09d8796eaed23afb71f5127d8cd3b1cdfb))
* **flake:** nix provided via NIX_CMD_PATH ([b478c9d](https://github.com/mobusoperandi/eelco/commit/b478c9db2ae22cbf52974b74ec268759023971fb))
* multiline results ([c8b068c](https://github.com/mobusoperandi/eelco/commit/c8b068c2c4b133580aed3cb74fca40d3837651e4))
* rewrite result mismatch error ([b2b0190](https://github.com/mobusoperandi/eelco/commit/b2b01907db72d61927226de2c8db0539009dd607))

### Bug Fixes

* wrapped nix ([56c90fe](https://github.com/mobusoperandi/eelco/commit/56c90fef8bca0f4bc3f0e530f789326643fc2fa7))

# Changelog

## [1.3.0](https://github.com/mobusoperandi/eelco/compare/v1.2.0...v1.3.0) (2024-02-22)


### Features

* no restriction on what expression examples eval into ([e458ca0](https://github.com/mobusoperandi/eelco/commit/e458ca0414feb2fd9d5c69c255cc7c39e21f5d6e))

## [1.2.0](https://github.com/mobusoperandi/eelco/compare/v1.1.1...v1.2.0) (2024-02-21)


### Features

* support skipping examples ([14d03a3](https://github.com/mobusoperandi/eelco/commit/14d03a34e6a4b81c642777d6549d9d46064c812c))

## [1.1.1](https://github.com/mobusoperandi/eelco/compare/v1.1.0...v1.1.1) (2024-02-21)


### Bug Fixes

* eprint example id when failing to parse it ([202e8ef](https://github.com/mobusoperandi/eelco/commit/202e8ef8ddd42556dfba326567579ba6d8067391)), closes [#88](https://github.com/mobusoperandi/eelco/issues/88)

## [1.1.0](https://github.com/mobusoperandi/eelco/compare/v1.0.1...v1.1.0) (2024-02-20)


### Features

* non-flake export using flake-compat ([756dcef](https://github.com/mobusoperandi/eelco/commit/756dcefc34ff3172f2d2666ef8ae3ce9d2f5bcfe))

## [1.0.1](https://github.com/mobusoperandi/eelco/compare/v1.0.0...v1.0.1) (2024-02-20)


### Bug Fixes

* skip tests in nix pkg ([44ab8ab](https://github.com/mobusoperandi/eelco/commit/44ab8abd675b1f3e1b5550f99a39b0aa74379f10))

## [1.0.0](https://github.com/mobusoperandi/eelco/compare/v0.2.0...v1.0.0) (2024-02-20)


### ⚠ BREAKING CHANGES

* repl example must have trailing blank line

### Features

* expression examples ([23a36fd](https://github.com/mobusoperandi/eelco/commit/23a36fd71059e15fd0f6526d8cc94a84b09468b2))
* support assignment in repl examples ([221f195](https://github.com/mobusoperandi/eelco/commit/221f195bad953d32966ff90431b81e503f06432f))


### Bug Fixes

* race between printing to stderr and terminating ([0fe24b8](https://github.com/mobusoperandi/eelco/commit/0fe24b8197bebbbb368db75a6fc4fb58b5f6f6c6))

## [0.2.0](https://github.com/mobusoperandi/eelco/compare/v0.1.0...v0.2.0) (2024-01-28)


### Features

* no debug fmt in result mismatch output ([9f7d700](https://github.com/mobusoperandi/eelco/commit/9f7d70018bf366e95d6c12dadba509ca507cfdfb))
* rm extraneous LF from result mismatch output ([51e9aa6](https://github.com/mobusoperandi/eelco/commit/51e9aa6296adf974d28e2bd6d14530d428d537bc))
