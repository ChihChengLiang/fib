# Fib Halo2

```
cargo run
```

See circuit error by

```diff
-let value = value_pre + value_prepre;
+let value = value_pre + value_prepre + F::one();
```

