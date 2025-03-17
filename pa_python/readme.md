# Python bindings to semi-global `pa_bitpacking::search`

This is a trivial python wrapper around the semi-global `search` function.

Initialize an environment using `just init_env`, and build using `just build`,
which needs `maturin`. Install maturin via `pipx install maturin`.

After `just build`, you can find the python package in `.env/lib[64]/python3.13/site-packages/pa_python`.

Then do `source .env/bin/activate` and use it like this:

```python
import pa_python
pa_python.search(b'CT', b"ACTG", 1.0)
```
which returns: `[2, 2, 1, 0, 1, 2, 2]`, which indicates the the pattern `CT`
matches best in the text `ACTG` when ending at the third character.

The pattern may also contain `N` or `*` to match any character of the text.

See the [rust documentation](../pa-bitpacking/src/search.rs) for more details.
