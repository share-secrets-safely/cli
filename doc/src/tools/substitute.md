```bash,prepare=in-fixtures,hide
cd doc/src/tools/fixtures/substitute
```
```bash,use=sy-in-path,exec
sy substitute --help
```

You can also use this alias: **sub**.
 
### Control your output

`template-specs` are the bread and butter of this substitution engine. They allow to not
only specify the input templates, like `./some-file.tpl`, but also set the output location.

By default, this is standard ouptut, but can easily be `some-file.yml`, as in `./some-file.tpl:out/some-file.yml`.

You can have _any amount of template specs_, which allows them to use the same, possibly
expensive, data-model.

#### Separating YAML Documents

At first sight, it might not be so useful to output multiple templates to standard output.
Some formats are built just for that usecase, provided you separate the documents correctly.

If there are multiple YAML files for instance, you can separate them like this:

```bash,use=in-fixtures,use=sy-in-path,exec=0
echo 'value: 42' \
| sy substitute --separator=$'---\n' <(echo 'first: {{value}}') <(echo 'second: {{value}}')
```

Also note the explicit newline in the separator, which might call for special syntax
depending on which shell you use.

#### Validating YAML or JSON Documents

In the example above, how great would it be to protect ourselves from accidentially creating
invalid YAML or JSON documents?

Fortunately, `sheesy` has got you covered with the `--validate` flag.

```bash,use=in-fixtures,use=sy-in-path,exec=1
echo 'value: 42' \
| sy substitute --validate <(echo '{"first":"{{value}}}') 
```

#### Protecting against 'special' values

When generating structured data files, like YAML or JSON, even with a valid template you
are very vulnerable to the values contained in the data-model. Some passwords for instance
may contain characters which break your output. Even though `--validate` can tell you right away,
how can you make this work without pre-processing your data?

`--replace` to the rescure. The following example fails to validate as the password was
now changed to contain a special character in the JSON context:

```bash,use=in-fixtures,use=sy-in-path,exec=1
echo 'password: xyz"abc' \
| sy substitute --validate <(echo '{"pw":"{{password}}"}') 
```

Here is how it looks like without validation:
```bash,use=in-fixtures,use=sy-in-path,exec=0
echo 'password: xyz"abc' \
| sy substitute <(echo '{"pw":"{{password}}"}') 
```

You can fix it by replacing all violating characters with the respective escaped version:

```bash,use=in-fixtures,use=sy-in-path,exec=0
echo 'password: xyz"abc' \
| sy substitute --replace='":\"' --validate <(echo '{"pw":"{{password}}"}') 
```

### How to use multi-file data in your templates

You have probably seen this coming from a mile away, but this is a great opportunity for a shameless plug to advertise `sy merge`.

`sy merge` allows to merge multiple files together to become one, and even some additional processing to it.
That way you can use the combined data as model during template substitution.

```bash,use=in-fixtures,use=sy-in-path,exec
sy merge --at=team team.yml --at=project project.yml --at=env --environment \
| sy substitute kubernetes-manifest.yaml.tpl
```

### Templates from STDIN ? Sure thing...

By default, we read the _data model_ from stdin and expect all templates to be provided
by `template-spec`. However, sometimes doing exactly the opposite might be what you need.

In this case, just use the `-d` flag to feed the _data model_, which automatically turns
standard input into expecting the template.

```bash,use=in-fixtures,use=sy-in-path,exec
echo '{{greeting | capitalize}} {{name}}' | sy substitute -d <(echo '{"greeting":"hello", "name":"Hans"}')
```

### Meet the engines

The substitution can be performed by various engines, each with their distinct advantages and disadvantages.

This section sums up their highlights.

#### Liquid (default)

The [`Liquid` template engine][liquid] was originally created for web-shops and is both easy to use as well as fully-featured.

It’s main benefit is its various filters, which can be used to put something into uppercase (`{{ “something” | uppercase }}`), or to encode text into base64 (`{{ “text” | base64 }}`).

There are a few filters which have been added for convenience:

* **base64**
	* Converts anything into its base64 representation.
	* No arguments are supported.

[liquid]: http://shopify.github.io/liquid/

#### Handlebars

The first optional template engine is [`handlebars`][hbs]. Compared to `Liquid`, it’s rather bare-bone and does not support any filters. The filtering syntax also makes chained filters more cumbersome.

However, it allows you to use _partials_, which are good to model something like multiple sites, which share a header and a footer. The shared portions are filled with data that contextually originates in the page that uses them.

For example, in an invocation like this you can declare headers and footers without rendering them, and then output multiple pages that use it.

_Here is the content of the files used_:

```bash,use=in-fixtures,exec
cat data.json
```

```bash,use=in-fixtures,exec
cat base0.hbs
```

```bash,use=in-fixtures,exec
cat template.hbs
```

When using these in substitution, this is the output:
```bash,use=in-fixtures,use=sy-in-path,exec
sy substitute --engine=handlebars -d data.json base0.hbs:/dev/null template.hbs
```

The perceived disadvantage of having close to zero available filters would have to be compensated using a processing program which takes the data, and adds all the variations that you would need in your templates:

```bash,use=in-fixtures,use=sy-in-path,exec
./data-processor < data.json | sy substitute template.tpl
```

The `data-processor` in the example just adds transformed values for all fields it sees:

```bash,use=in-fixtures,use=sy-in-path,exec
./data-processor < data.json
```

[hbs]: http://handlebarsjs.com
