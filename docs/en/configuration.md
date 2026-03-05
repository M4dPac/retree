# ⚙️ Configuration

> Configuration file support is not available in the current version.  
> Support for `~/.rtreerc.toml` is planned for future releases.

---

## Environment variables

| Variable      | Values       | Description                                      |
| ------------- | ------------ | ------------------------------------------------ |
| `TREE_COLORS` | color string | Color settings (takes priority over `LS_COLORS`) |
| `LS_COLORS`   | color string | Color settings in GNU ls format                  |
| `TREE_LANG`   | `en` or `ru` | Interface language (same as `--lang`)            |
| `NO_COLOR`    | any value    | Disables all colors when set                     |

### Setting variables

**Linux / macOS:**

```bash
export TREE_LANG=en
export TREE_COLORS="di=1;34:ex=1;32:*.rs=1;33"
export NO_COLOR=1
```

**Windows PowerShell:**

```powershell
$env:TREE_LANG = "en"
$env:TREE_COLORS = "di=1;34:ex=1;32:*.rs=1;33"
$env:NO_COLOR = "1"
```

**Windows cmd:**

```cmd
set TREE_LANG=en
set TREE_COLORS=di=1;34:ex=1;32:*.rs=1;33
```

---

## Settings priority

CLI flags always override environment variables.

```
CLI flags  >  TREE_COLORS / TREE_LANG  >  LS_COLORS  >  defaults
```

---

## More about colors

The `TREE_COLORS` / `LS_COLORS` string format is described in [colors.md](colors.md).
