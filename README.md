# installation

fish shell

```fish
function km
        set -l __km_result $($KM_BINARY_PATH 2>&1 >/dev/tty)
        if string length -q -- "$__km_result"
        cd $__km_result
    else
        echo "Path not found."
    end
end
```
