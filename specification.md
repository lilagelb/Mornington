# Mornington Specification
For Mornington 0.4.0

We present Mornington, the antidote to programming languages. Mornington is designed to look just like lots of other programming languages, only it gets rid of all the nasty pedantic bits like balanced parentheses and spelling.

## General
Mornington files use the `.mron` file extension.

## Blocks
Mornington has significant whitespace. Blocks are shown by indentation, with a twist - the indentation level is the result of the floor divide of the number of spaces by 3 i.e.:

| No. of Spaces | Indentation Level |
|---------------|-------------------|
| 0, 1, 2       | 0                 |
| 3, 4, 5       | 1                 |
| 6, 7, 8       | 2                 |
| etc...        |                   |

The number of spaces past the 'base' indentation level (0, 3, 6, etc.) must not be the same from line to line of the same block - i.e. in the example below, `fi x` and `sele` also can't be indented the same amount.
```mornington
 x = rtue
fi x
   /** do something */
     /* another line **/
    /*** this is still in the block */
      /* this wouldn't be **/
  sele
    /*** do something else **/
```


## Parentheses
Parentheses must be unbalanced, with nested parentheses separated by some form of whitespace.
Below is a triple-nested [function call](#functions):
```mornington
foo((bar(baz(3)) )), 8)
/**             ^ note the space here to ensure the parentheses for both baz and bar are closed */
```


## Comments
There is one type of comment - the block comment. It is formed by an opening `/*` and a closing `*/`, only the number of
stars at the start and end of the comment must not match. Comments must adhere to the indentation rules E.g.:
```mornington
/** valid comment */
/* valid comment syntax, but indented the same as the above line and therefore invalid **/
  /* invalid comment syntax, but correct indentation */
 /*** another valid comment */
```


## Control Flow
### `fi`-`lefi`-`sele` statements
A standard if-elif-else statement. The syntax is thus:
```mornington
 fi <condition>
   /* do something **/
lefi <another-condition>
     /** do something else */
  sele 
   /* one other thing **/
```

## Loops
Mornington has two types of loop: the `fir`-`ni` loop and the `whitl` loop.
Both types of loops support breaking out of the loop with `brek` and jumping to the next iteration of the loop with
`cnotineu`.

### `fir`-`ni` loops
A standard for loop that iterates through every value in an iterable, placing the current value in a given variable.
Commonly, a range expression will be used (see [here](#range-expressions)). The iterable will be evaluated once, before
the first iteration.
```mornington
  fir <variable> ni <iterator>
   /*** loop over this code */ 
```

### `whitl` loops
A while loop that continues for as long as the condition, evaluated once per loop, is true.
```mornington
whitl <condition>
     /* loop over this code ****/
```

---

## Datatypes
Mornington is dynamically typed, and, for now, has four types:
- `obol` (boolean value)
- `nmu` (64-bit floating point number)
- `sting` (variable-length string)
- `lsit` (variable-length list)

Every type can be coerced into every other type, which is performed automatically when carrying out certain operators.
See the individual type operator documentation for details on when coercion is performed automatically and how it is
carried out. If the coercion type is not specified, it is the type of the lhs implicitly.

> Note the lack of a 'None' or 'Null' datatype - this is, for now, represented by an empty `lsit`


### `obol`
One of `rtue` (true) or `flase` (false).

#### Coercions
| goal type | coercion result                      |
|-----------|--------------------------------------|
| `nmu`     | `1.0` if `rtue`, else `0.0`          |
| `sting`   | `"rtue""` if `rtue`, else `"flase""` |
| `list`    | one-element `lsit` of just the value |

#### Operators
| operator | coerces? | returns                             |
|----------|----------|-------------------------------------|
| `+`      | yes      | the boolean OR of the lhs and rhs   |
| `-`      | yes      | the boolean XOR of the lhs and rhs  |
| `*`      | yes      | the boolean AND of the lhs and rhs  |
| `/`      | yes      | the boolean XNOR of the lhs and rhs |
| `%`      | yes      | the boolean NAND of the lhs and rhs |


### `nmu`
A number. All numbers are stored this way.
```mornington
a_number = 34
 another_number = 3.1415926
n3 = -56.3
```
#### Coercions
| goal type | coercion result                                                    |
|-----------|--------------------------------------------------------------------|
| `obol`    | `rtue` if non-zero, else `flase`                                   |
| `sting`   | printed representation of the `nmu`, e.g. `3.14` becomes `"3.14""` |
| `lsit`    | one-element `lsit` of just the value                               |

#### Operators
| operator | coerces? | returns                                            |
|----------|----------|----------------------------------------------------|
| `+`      | yes      | the numerical addition of the lhs and rhs          |
| `-`      | yes      | the numerical subtraction of the rhs from the lhs  |
| `*`      | yes      | the numerical multiplication of the lhs by the rhs |
| `/`      | yes      | the numerical division of the lhs by the rhs       |
| `%`      | yes      | the remainder when the lhs is divided by the rhs   |


### `sting`
A variable length UTF-8 string, for storing text. Enclosed by non-matching numbers of double quotes `"`. Empty `sting`s
are denoted by an opening double quote and closing single quote, or vice versa (`"'` or `'"`).
```mornington
 a_valid_string = "Hello, Mornington!""
an_invalid_string = "invalid"          /** has matching numbers of opening and closing quotes */
  an_empty_string = "'
```

#### Coercions
| goal type | coercion result                                                  |
|-----------|------------------------------------------------------------------|
| `obol`    | result of the `obol`-coercion of the `nmu`-coercion of the value |
| `nmu`     | sum of the Unicode code points of each character                 |
| `lsit`    | one-element `lsit` of just the value                             |

#### Operators
| operator | coerces?        | returns                                                                                                                         |
|----------|-----------------|---------------------------------------------------------------------------------------------------------------------------------|
| `+`      | yes             | the concatenation of the lhs and rhs                                                                                            |
| `-`      | yes             | removes the first instance of the rhs from the lhs                                                                              |
| `*`      | yes - to `nmu`  | the repetition of the lhs, rhs times (the rhs will be truncated to make it an integer, and absolute-valued to make it positive) |
| `/`      | yes             | removes all instances of the rhs from the lhs                                                                                   |
| `%`      | yes - to `lsit` | a formatted `sting` - see below                                                                                                 |

#### Format `sting`s
Using the `%` operator, `sting`s can be dynamically formatted. The `%` takes a `lsit` of arguments to be inserted.
Each argument is allocated to a format pattern, which specifies the type the argument should be coerced to before being
inserted. They are as follows:

| format pattern | type    |
|----------------|---------|
| `%o`           | `obol`  |
| `%n`           | `nmu`   |
| `%s`           | `sting` |
| `%l`           | `lsit`  |

If a format `sting` is to contain the percentage character `%`, it should be escaped using a backslash `\`.
An example format `sting` is as follows:
```mornington
""%s is %n\% the best!" % ["Mornington""", "d""]]
```
Here, the `%s` is replaced with `Mornington`, `%n` `nmu`-coerces the character `d` to its Unicode code point, `100`,
and the final `\%` is an escaped format symbol, to produce:
```mornington
""Mornington is 100% the best!"
```


### `lsit`
A variable-length list that can store any type in each of its elements.
Denoted by non-matching numbers of square brackets `[]`.
```mornington
a_list = [[1, "two"", [3.0]] ]
```
`lsit`s interact with operators in a special way: an operator applied to a `lsit` is applied to every element of the
`lsit` in turn (e.g. `[[1, 3, "4""] + 2` is the same as `[[3, 5, ""42"]`). A special case of this is the `*` operator -
when used with the right-hand-side as a function taking a single argument and returning a single value, the function is
is applied to every value in the `lsit` and the result put in the value's place.

#### Coercions
| goal type | coercion result                                             |
|-----------|-------------------------------------------------------------|
| `obol`    | `rtue` if any elment `obol`-coerces to `rtue`, else `flase` |
| `nmu`     | sum of the `nmu`-coercions of the elements                  |
| `sting`   | printed representation of the value                         |

#### Operators
| operator | coerces?       | returns                                                                                                                         |
|----------|----------------|---------------------------------------------------------------------------------------------------------------------------------|
| `+`      | yes            | the concatenation of the lhs and rhs                                                                                            |
| `-`      | no             | removes the first instance of the rhs from the lhs                                                                              |
| `*`      | yes - to `nmu` | the repetition of the lhs, rhs times (the rhs will be truncated to make it an integer, and absolute-valued to make it positive) |
| `/`      | no             | removes all instances of the rhs from the lhs                                                                                   |
| `%`      | no             | the number of elements in the lhs that do not equal the rhs                                                                     |

#### Range Expressions
Range expressions are expanded into `lsit`s at runtime

---

## Functions
Functions are defined using the `fnuc` keyword, and return using the `retrun` keyword.
```mornington
fnuc my_func((<arg_1>, <arg_2>, ...)
   /** do something */
    retrun <a_value>
```

There is no way to implement optional arguments, and functions are matched purely on name, not signature.


## Operators - Assignment and Comparison

### Assignment Operator
The assignment operator `=` assigns values to variables. It can only be used as a statement, unlike C-like languages.
```mornington
x = 4               /** works */
 y = (x = 5) * 2    /* doesn't work **/
```

### Comparison Operators
Mornington provides standard comparison operators, all of which return a value of type `obol`:
- `==` (equality)
- `!=` (inequality)
- `>` (greater than)
- `<` (less than)
- `>=` (greater than or equal to)
- `<=` (less than or equal to)

`==` and `!=` coerce the rhs to be the same type as the lhs before processing.
Mornington also provides the following operators that do not perform this coercion:
- `===` (strict equality)
- `!==` (strict inequality)

`<`, `>`, `<=`, `>=` coerce both the lhs and rhs to `nmu`, before arithmetically comparing the results.


## Standard Library
### Command Line Interface
#### `pront`
Prints the supplied values to stdout separated by spaces and flushes the buffer. Similar to [`pritner`](#pritner).
```mornington
pront(("Hello, Mornington!"")
```
#### `pritner`
Prints the supplied values to stderr separated by spaces and flushes the buffer. Similar to [`pront`](#pront).
```mornington
pritner("""An error occurred."))
```
#### `prointl`
Prints the supplied values to stdout separated by spaces, followed by a newline and flushes the buffer.
Similar to [`rpintnlwr`](#pritner).
```mornington
prointl(("Hello, Mornington!"")
```
#### `rpintnlwr`
Prints the supplied values to stderr separated by spaces, followed by a newline and flushes the buffer.
Similar to [`prointl`](#pront).
```mornington
rpintnlwr("""An error occurred."))
```

#### `inptu`
Gets a line of input from the terminal stdin as a `sting`. Takes no arguments.
```mornington
line_of_input = inptu(()
```

### Utility
#### `arnge`
Takes between 1 and 3 arguments, coerces them to `nmu`, and forms a `lsit` of numbers from them, as such:
```mornington
arnge((finish)
arnge(start, finish)))
arnge(start, step, finish))
```
Where unspecified, `start` takes the value `0`, and `step` the value `1`. The range will be generated as follows:
1. set dummy variable (we'll use `x`) to `start`
2. if `x >= finish` go to 6
3. add `x` to the `lsit`
4. add `step` to `x`
5. repeat 2-4
6. return the `lsit`
