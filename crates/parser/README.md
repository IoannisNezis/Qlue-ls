<h1 align="center">
    LL-SPARQL-parser
</h1>

A resilient LL Sparql parser.  
This is the parser that powers [Qlue-ls](https://github.com/IoannisNezis/Qlue-ls), a SPARQL Langauge server.

It uses [rowan](https://github.com/IoannisNezis/rowan) for the
[red-green-tree](https://ericlippert.com/2012/06/08/red-green-trees/) datastructure under the hood.  
The produced syntax trees are defined in [sparql.ungram](sparql.ungram).

The Parser is generated from the grammar and provides a lossless concrete syntax tree.  
This tree can become quiet nested and inconvient for programatic access.  
To encounter this problem the ast module provides a convient access functions.
