# bmrs dsl grammar

a program is a set of function definitions...

```txt
<program>   ::= <def>*

<def>       ::= "let" HASH? IDENT "(" <params> ")" "=" <expr>
<params>    ::= IDENT ( "," IDENT )*                        // one or more distinct parameters

<expr>      ::= <or_expr>
<or_expr>   ::= <and_expr> ("or" <and_expr>)*
<and_expr>  ::= <not_expr> ("and" <not_expr>)*
<not_expr>  ::= "not" <atomic>
              | <atomic>

<atomic>    ::= "if" <expr> "then" <expr> "else" <expr>
              | IDENT "(" <expr> ( "," <expr> )* ")"        // application
              | "(" <expr> ")"                              // grouping
              | IDENT                                       // variable
              | "true"
              | "false"
```
