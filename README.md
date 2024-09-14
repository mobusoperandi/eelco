![eelco](./eelco.png)

This is a CLI that reads markdown files,
extracts Nix examples from them and
tests these examples.

For CLI docs see the output with the `--help` flag.

Two kinds of examples are supported:

1. expression examples
2. repl examples

Expression examples look like this:

````md
```nix
1 + 1
```
````

They are nix expressions inside of fenced code blocks.
The first word in their info string is `nix`.
The expression is passed to Nix for evaluation as `nix-instantiate --expr --eval <EXPRESSION>`.
Tip: use assertions to prove intermediate values, where appropriate.

If a `nix` fenced code block is followed immediately by 
Example:

````md
```nix
1 + 1
```

```nix expected-value
2
```
````

Repl exaples look like this;

````md
```nix-repl
nix-repl> a = 1

nix-repl> a + 1
2

```
````

Repl examples will be evaluated by the real Nix repl.
At the moment, expressions and assignments queries are supported.
A line that follows an expression query will be used as an assertion.
Blank lines matter. Even trailing ones.

Examples can be skipped by including the word `skip` in the info string.

The name eelco is in homage to the original author of Nix, Eelco Dolstra.
