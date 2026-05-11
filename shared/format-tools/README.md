# ASEZ Formatting Tools

## numeric_format

A macro for selecting the correct grammatical form of a word based on a numeral.
Uses [`shared_essential::application::message::numeral_relation`].

### Usage

```rust,ignore
let count = 123;
let name = "PD";
let res = numeric_format!(
    "{@count} {@route|routes|routes_plural} {name} of type {ty} were deleted",
    ty = 2
);
assert_eq!(res, "123 routes PD of type 2 were deleted");
```

Or with positional arguments:

```rust,ignore
let count = 123;
let another_var = count;
let name = "PD";
let res = numeric_format!(
    "{} {@route|routes|routes_plural} {name} of type {ty} were deleted",
    another_var, @count, ty = 2
);
assert_eq!(res, "123 routes PD of type 2 were deleted");
```

When listing format parameters, prefix the numeral parameter with `@` — this is the number
used to select the correct word form.

- `{@singular|within_2_and_4|default}` — calls `get_controlled_numeric(count, singular, within_2_and_4, default)`.
- `{@singular|plural}` — calls `get_conformed_numeric(count, singular, plural)`.

## fomat

A wrapper around `format!` for more flexible string formatting.

### Motivation

In HTML, `{` and `}` are used in CSS class styles such as:

```html
<html lang="en">
  <body>
    <div class="super_class">Something</div>
  </body>
  <style>
    .super_class {
      background-color: green;
    }
  </style>
</html>
```

This means every `{` and `}` in the template string would have to be escaped as `{{` and `}}`.
HTML templates should not depend on our internal message generation implementation.

### Usage

You can configure delimiters other than `{` and `}`. For example:

```rust
let res = fomat!(
    "[-",
    "-]",
    "Hello, [-name-]. Have a good day {hopefully}",
    name = "John"
);

assert_eq!(res, "Hello, John. Have a good day {hopefully}");
```
