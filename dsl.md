# bmrs dsl grammar

a program is a set of function definitions...

```txt
<program>   ::= <def>*

<def>       ::= "let" HASH? IDENT "(" <params> ")" "=" <expr>
<params>    ::= IDENT ( "," IDENT )*                        // one or more distinct parameters

<expr>      ::= "if" <expr> "then" <expr> "else" <expr>
              | IDENT "(" [ <expr> ( "," <expr> )* ] ")"    // application
              | "(" <expr> ")"                              // grouping
              | IDENT                                       // variable
              | "true"
              | "false"
```
