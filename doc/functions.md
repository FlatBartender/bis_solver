# Functions

### Weapon delay

Not much to see here, it's just a shortcut for SGE weapons. This isn't used anywhere currently.

```rust,ignore
{{#include ../src/data.rs:21:23}}
```

### GCD 

(sources: [1](https://www.akhmorning.com/allagan-studies/how-to-be-a-math-wizard/shadowbringers/speed/) and [2](https://www.akhmorning.com/allagan-studies/stats/speed/))

GCD is a `Unit<1, 100>`.

```rust
{{#include ../src/data.rs:25:27}}
```

There is a small difference with the sources' formulas. Mainly, instead of *adding* `130 * ceil(400 - sps)`, `130 * (sps - 400)` is subtracted.
Since All operation are on integers, a floor is automatically applied, and the operations are identical.

In addition, SGE doesn't have any GCD modifiers outside of spellspeed so the typeX and typeY modifiers have been removed.

In addition to GCD, GCD15 exists for 1.5s based GCDs:

```rust,ignore
{{#include ../src/data.rs:29:31}}
```

### Critical hit

```rust,ignore
{{#include ../src/data.rs:41:43}}
```

```rust,ignore
{{#include ../src/data.rs:37:39}}
```

```rust,ignore
{{#include ../src/data.rs:73:75}}
```

`crit_factor` is not a `Unit`, because it isn't tied to the integer rules. It's a factor that integrates the averages critical damage and non-critical damage according to their weights.

### Direct hit

```rust,ignore
{{#include ../src/data.rs:49:51}}
```

```rust,ignore
{{#include ../src/data.rs:77:79}}
```

Like `crit_factor`, `dh_factor` is not a unit because it is not used in integer math formulas and only in damage estimation formulas.

### Determination

```rust,ignore
{{#include ../src/data.rs:45:47}}
```

### Spell speed DoT multiplier

```rust,ignore
{{#include ../src/data.rs:53:55}}
```

### Adjusted weapon damage

```rust,ignore
{{#include ../src/data.rs:57:59}}
```

This is hardcoded for SGE.

### Attack power

```rust,ignore
{{#include ../src/data.rs:65:67}}
```

### Trait bonus

```rust,ignore
{{#include ../src/data.rs:69:71}}
```

