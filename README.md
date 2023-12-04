# Gear onchain tests tools

...Still in development...


## How to use

#### Install command-line tool:

```
cargo install --git https://github.com/NikVolf/onchain-tests
```

This will add `gear-ot` binary to your path!


#### Describe tests in your wasm by depending on `onchain-tests-types` and declaring fixtures in `test` function:

```
EXAMPLE COMING
```

or declare programmatic tests in that function:

```
EXAMPLE COMING
```

#### Upload your instance of testing service according to your test plan decribed above:

```
gear-ot deploy <your_project_name.test.wasm>
```

to get your testing service program id (`PROGRAM_ID`)


#### Finally, run fixtures onchain
```
gear-ot <PROGRAM_ID>
```

Feel free to insert it in your test infrastructure from now on
