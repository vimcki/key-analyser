# Key-analyser

This is histogram generator for keypress data gathered with:
```
xinput test-xi2 --root | grep --line-buffered -A 3 -E '\(KeyPress\)' | grep --line-buffered detail | awk '{print $2;fflush()}'
```
