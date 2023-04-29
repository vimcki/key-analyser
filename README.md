# Key-analyser

This is histogram generator for keypress data gathered with:
```
xinput test-xi2 --root | grep --line-buffered -A 3 -E '\(KeyPress\)' | grep --line-buffered detail | awk '{print $2;fflush()}'
```

it calls `xmodmap -pke` underneath to get keycode -> text representation

## Usage

You can either give it file paths or pipe stdin
```
$ file keylog.txt
keylog.txt: ASCII text

$ cargo run keylog.txt
...

$ cat keylog.txt | cargo run
...
```
