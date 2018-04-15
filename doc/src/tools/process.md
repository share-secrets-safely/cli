```bash,use=sy-in-path,prepare=in-fixtures,hide
cd doc/src/tools/fixtures/process
```
```bash,use=sy-in-path,exec
sy process --help
```

You can also use these aliases:
 * **merge**
 * **show**

It helps to use this powerful command by understanding its mindset a little.

 * it wants to produce a single document _(JSON or YAML)_ from multiple input documents _(JSON or YAML)_
 * by default, it will *refuse to overwrite* existing values
 * multi-document YAML files are fully supported
 * standard input is a valid source for documents
 * the order of arguments matter, as this program is implemented as *state-machine*
 * by default it produces *JSON* output

This program helps to quickly manipulate various inputs and produce a new output, which can then
more easily be used in programs like `sy substitute`, or to generate configuration files.

Now let's look typical use-cases.

### Merge multiple documents into one

This case often arises by the mere necessity of keeping things separate. Even though
keeping all data in a single structured data file would work just fine, in practice
not all information is under your direct control and thus pulled in separately from
other locations.

For substitution, multiple files are not viable, which is why a single file should be produced
instead:

```bash,use=in-fixtures,exec
sy merge --at=project project.yml --at=team team.yml
```

As you can see, we use the `--at` flag to put the contents of both files into their
own namespaces. Without that, we would have a clashing `name` field which makes the
program fail.

```bash,use=in-fixtures,exec=1
sy merge project.yml team.yml
```

### Overwriting individual values (or creating new ones)

Sometimes during testing, it's useful to change a single value, in the configuration
for instance. You can easily do this using the `key=value` specification.

```bash,use=in-fixtures,exec=1
sy merge game-config.yml player.lives=99
```

However, the above fails as we won't ever overwrite existing values. Let's try to
argue with the program to make it work nonetheless:

```bash,use=in-fixtures,exec=1
sy merge game-config.yml player.lives=99 --overwrite
```

This might appear unexpected, even though it is not when you have understood that the order
matters. In the example above, `--overwrite` simply applies too late for overwriting the
value. If we swap their positions, it will work.

```bash,use=in-fixtures,exec
sy merge game-config.yml  --overwrite player.lives=99 player.invincible=true
```

Please note that `--overwrite` acts as a toggle and affects all following arguments. You can
toggle it back off with the `--no-overwrite` flag

```bash,use=in-fixtures,exec
sy merge game-config.yml  --overwrite player.lives=99 \
        --no-overwrite --at=project project.yml --at=team team.yml \
        -o yaml
```

### Converting YAML to JSON (or vice versa)

As a side-effect, you can easily convert YAML to JSON, like so...

```bash,use=in-fixtures,exec
sy process < game-config.yml
```

...or the other way around:

```bash,use=in-fixtures,exec
sy process < game-config.yml | sy process -o yaml
```

### Accessing to environment variables

More often than not, the environment contains information you will want to make use of
in our configuration. It's easy to bring it into your data model, and filter them by their name.

```bash,use=in-fixtures,exec
sy process --environment=HO*
```

Of course this can be combined with all other flags:

```bash,use=in-fixtures,exec
sy process --at env --environment=HO* env.NEW=value
```

### 'Pulling up' values to allow general substitution

It's most common to have different sets of configuration for different environments. For example, most
deploy to at least two stages: *pre-production* and *production*.

When using `sy process` for generating configuration to be used by tooling, it's not practical to force
the tooling to know the stage.

Imagine the following configuration file:

```bash,use=in-fixtures,exec
cat multi-stage-config.yml
```

Tools either get to know which stage configuration to use, or you 'pull it up' for them:

```bash,use=in-fixtures,exec
sy process --select=production multi-stage-config.yml -o yaml
```

### Using multi-document yaml documents as input

A feature that is still rare in the wild, probably due to lacking tool support, is multi-document
YAML files.

We fully support them, but will merge them into a single document before processing it any further.

A file like this...
```bash,use=in-fixtures,exec
cat multi-document.yml
```

...looks like this when processing. Clashing keys will clash unless `--overwrite` is set.
```bash,use=in-fixtures,exec
sy process multi-document.yml -o yaml
```

### Controlling standard input

We will read JSON or YAML from standard input if possible. To make it more flexible,
any *non-path* flags are applied to standard input. This may lead to unexpected output
if more than one document source is specified.

Let's start with a simple case:

```bash,use=in-fixtures,exec
cat team.yml | sy process --at=team-from-stdin
```

In the moment another file is added for processing, it's a bit more difficult to
control which argument applies where:

```bash,use=in-fixtures,exec
cat team.yml | sy process --at=team-from-stdin --at=project project.yml
```

As you can see, the `project` key is used for standard input, and the `team-from-stdin`
is seemingly ignored.

To fix this, be explicit to make obvious what you expect:

```bash,use=in-fixtures,exec
cat team.yml | sy process --at=team-from-stdin - --at=project project.yml
```

Note the single `dash (-)`, which indicates when to read from standard input. As
standard input is always consumed entirely, it can only be specified *once*.
