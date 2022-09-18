# Units

In the Allagan Studies documents, an inordinate number of flooring operations are used.
They can be mostly eliminated (or at least kept under the rug) if we use integer math instead.

To this end, most calculations involving integers are made using Units.
A `Unit` is an integer associated with a numerator and a denominator

For example, this snippet
```rust,ignore
Unit<1, 100>(50)
```
represents `0.5` (the unit's "scalar").

Integers multiplications with a `Unit` is done this way :
```rust,ignore
{{#include ../src/utils.rs:15}}
```
Where `self` is the integer, `unit.0` is the unit's intrinsic integer, and `NUMERATOR` and `DENOMINATOR` are those of the unit.

This way, flooring is automatically done as we work on integers only.

# A bit of Rust syntax

This is the code representing the weapon delay of SGE.

```rust,ignore
{{#include ../src/data.rs:21:23}}
```

As you can see, the weapon delay is just a `Unit(280)`. Rust, the language used in this tool, will automatically infer some of the type parameters.

Without going into details, in line 1, we want to return a `Unit<1, 100>`; as such, the `Unit(280)` line 2 is automatically inferred to be `Unit<1, 100>(280)`.

