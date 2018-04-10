
```bash,use=sy-in-path,exec
sy process --help
```

### Caveats
 * You should use the `--option=arg` form for all named arguments. Otherwise the argument parsing
   may misinterpret your input for two positional arguments, and yield confusing results.

### Tips and Tricks (WIP)

 * Show behaviour with multi-document YAML files from file or stdin
 * Show overrides of values in the structured data
 * How to control position of stdin
