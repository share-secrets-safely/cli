[![Build Status](https://travis-ci.org/Byron/share-secrets-safely.svg?branch=master)](https://travis-ci.org/Byron/share-secrets-safely)

`share-secrets-safely` is a GPG based solution for managing shared secrets.

## On our way to the minimal viable product v1.0

 * [ ] **setup rust workspace with libraries for**
   * [x] CLI
   * [ ] GPG interface
   * [ ] YAML tools
   * [ ] first library publish to crates.io
 * [ ] **setup Travis CI for**
   * [ ] linux
   * [ ] OSX
 * [ ] **standalone deployables**
   * [ ] static linking on OSX
   * [ ] MUSL and static linking on linux
  
 * ...and many more when the boxes above are ticked :D

## Project Goals

 * **a great user experience**
   * The user experience comes first when designing the tool, making it easy for newcomers while providing experts with all the knobs to tune
   * deploy as *single binary*, no additional executables or dependencies are required to use all of the features
 * **proven cryptography**
   * Don't reinvent the wheel, use *gpg* for crypto
   * Thanks to *GPG* each user is identified separately through their public key
 * **automation and scripting is easy**
   * storing structured secrets is as easy as making them available in shell scripts
   * common operations like substituting secrets into a file are are natively supported
   * proper program exit codes make error handling easy
 * **user management**
   * support small and large teams, as well as multiple teams, with ease
   * make use of gpg's *web of trust* to allow inheriting trust even across team boundaries, and incentivize thorough checking of keys
 * **basic access control**
   * partition your secrets and define who can access them
 

## Non-Goals

 * **replicate `pass` or `gpg` functionality directly**
   * having seen what `pass` actually is and how difficult it can be to use it especially in conjunction with `gpg`, this project will not even look at the provided functionality but be driven by its project goals instead.
 * **become something like hashicorp vault**
   * this solution is strictly file based and *offline*, so it can fill be used without any additional setup.


## Development Practices

 * **test-first development**
   * protect against regression and make implementing features easy
   * user docker to test more elaborate user interactions
 * **safety first**
   * handle all errors, never unwrap
   * provide an error chain and make it easy to understand what went wrong.
 * **strive for an MVP and version 1.0 fast...**
   * ...even if that includes only the most common usecases.
 * **Prefer to increment major version rapidly...**
   * ...instead of keeping major version zero for longer than needed.



