# jset
jset is a command-line tool for performing set operations on a list of json files

## Installation

Currently you will need a [Rust development environment setup](https://www.rust-lang.org/tools/install) to install `jset`.

```
cargo install jset
```

## Whirlwind Tour

`jset` requires at least two json files to work with, but can work with any number of json files. The basic structure of a jset command is as follows (where `FILE` is a path to a JSON file):

```
jset <operation> [flags] FILE FILE [...FILE]
```

By default all output is pretty printed, but this can be disabled using the `-c` flag. This will be done in all examples below to make them more readable.

### Union
`jset union` computes the set union between two or more json files.

#### Objects

Fields are recursively merged together.
```
$ jset union -c <(echo '{"a":{"b": "c"}}') <(echo '{"a":{"c": "d"},"e":"f"}')
{"a":{"b":"c","c":"d"},"e":"f"}
```

When conflicting keys exist, later files take precedence over earlier files.
```
$ jset union -c <(echo '{"a":"b"}') <(echo '{"a":"c"}')
{"a":"c"}
```

#### Arrays
Elements from each array are concatenated together.
```
$ jset union -c <(echo '["a","b","c"]') <(echo '["d","e","f"]')
["a","b","c","d","e","f"]
```

Duplicate elements from later lists are ignored in final result.
```
$ jset union -c <(echo '["a","b","c"]') <(echo '["b","c","d"]')
["a","b","c","d"]
```

#### Primitives (numbers, bools, strings, null)

Primitive values union successfully if the values are identical, otherwise no union is found and the program execution fails.
```
$ jset union -c <(echo '"a"') <(echo '"a"')
"a"
```

```
$ jset union -c <(echo '"a"') <(echo '"b"')
$ echo $?
1
```

### Intersect
`jset intersect` computes the set intersection between two or more json files.

#### Objects

New object is returned with fields that exist in all files, recursively.
```
$ jset intersect -c <(echo '{"a":{"b":"c","g":"i"}}') <(echo '{"a":{"b":"c","g":"h"},"e":"f"}')
{"a":{"b":"c"}}
```

#### Arrays
Elements from first file are retained as long as they exist in all subsequent files.
```
$ jset intersect -c <(echo '["a","b","c"]') <(echo '["a","d","e"]')
["a"]
```

#### Primitives (numbers, bools, strings, null)

Primitive values intersect successfully if the values are identical, otherwise no intersection is found and the program execution fails.
```
$ jset intersect -c <(echo '"a"') <(echo '"a"')
"a"
```

```
$ jset intersect -c <(echo '"a"') <(echo '"b"')
$ echo $?
1
```

### Difference
`jset difference` (alias: `jset diff`) computes the set difference between two or more json files. This is done iteratively with the 2nd file being subtracted from the first, the 3rd being subtracted from the difference between the 1st and 2nd and so on.

An example with four files would be processed in the following way:
```
((1-2)-3)-4

# or more simply
1-2-3-4
```

#### Objects

The returned object is the first file with the second file subtracted from it, recursively. Uknown fields in the subsequent files are ignored.
```
$ jset diff -c <(echo '{"a":{"b":"c","g":"i"}}') <(echo '{"a":{"b":"c","g":"h"},"e":"f"}')
{"a":{"g":"i"}}
```

#### Arrays
Elements from first file are removed if they appear in any subsequent files.
```
$ jset diff -c <(echo '["a","b","c"]') <(echo '["a","d","e"]')
["b","c"]
```

#### Primitives (numbers, bools, strings, null)

Performing set differences on primitive values will return the first value when all other values are not equal, otherwise it will fail.

> Note: I didn't have any use-case for doing set differences on primitives so I don't know if this is really useful behavior. :shrug:

```
$ jset diff -c <(echo '"a"') <(echo '"a"')
$ echo $?
1
```

```
$ jset diff -c <(echo '"a"') <(echo '"b"')
"a"
```
