This is a CLI that reads markdown files,
extracts Nix examples from them and
tests these examples.

Two kinds of examples are supported:

1. expression examples
2. repl examples

Expression examples look like this:

````md
```nix
assert 1 + 1 == 2; null
```
````

They are nix expressions inside of fenced code blocks.
The first word in their info string is `nix`.
The expression is passed to Nix for evaluation as `nix eval --expr <EXPRESSION>`.
For future compatibility, to be considered passing,
it must successfully evaluate into `null`.
It is expected of the author to demonstrate and prove their points
using assertions.

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
