
```bash,use=sy-in-path,exec
sy substitute --help
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

```bash,
sy substitute -d ./data.json ./header.tpl:/dev/null ./footer.tpl:/dev/null page-1.tpl:page1.html page-2.tpl:page2.html
```

**TODO** actual executable example showing the partial, and to documentation

The perceived disadvantage of having close to zero available filters would have to be compensated using a processing program which takes the data, and adds all the variations that you would need in your templates:

```bash
data-processor < data.json | sy substitute template.tpl
```

[hbs]: http://handlebarsjs.com

### How to use multi-file data in your templates

You have probably seen this coming from a mile away, but this is a great opportunity for a shameless plug to advertise `sy merge`.

`sy merge` allows to merge multiple files together to become one, and even some additional processing to it.
That way you can use the combined data as model during template substitution.

```bash
sy merge ./etc/ext/team.yml ./etc/project.yml —at=environment -e | sy substitute template.tpl
```

### Tips and Tricks (WIP)

 * When data is provided via the `-d` flag, everything from stdin is interpreted as template.
