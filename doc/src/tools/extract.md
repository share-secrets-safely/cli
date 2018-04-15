```bash,use=sy-in-path,prepare=in-fixtures,hide
cd doc/src/tools/fixtures/extract
```
```bash,use=sy-in-path,exec
sy extract --help
```

You can also use this alias: **fetch**.

The `extract` sub-command is meant for those cases when you need individual values from
a file with structured data, for example when consuming credentials in scripts.

When extracting scalar values, those will be output *one per line*. If an extracted value
*is not scalar*, e.g. an object or array, the output mode of all extracted values will change
to JSON (default) or YAML with `--output`.

All values are specified using [JSON-pointer notation][JSON-pointer], with the added convenience
that slashes _(-)_ can be exchanged with dots _(.)_ .

[json-pointer]: http://rapidjson.org/md_doc_pointer.html

### Extracting username and password

Given an input file like this, here is how you can extract `username` and `password` for usage
in scripts:

```bash,use=in-fixtures,exec
cat credentials.yml
```

Extract using the JSON pointer syntax is quite straightforward, and rather forgiving:

```bash,use=in-fixtures,exec
sy extract -f=credentials.yml user.name /user/password
```

From here it should be easy to assign individual values to variables for use in scripts

```bash,use=in-fixtures,exec
password="$(sy extract user.password < credentials.yml)"
username="$(sy extract user.name < credentials.yml)"
echo Authorization: Basic $(echo $username:$password | base64)
```

### Collecting individual values into a structured format

By default, and as long as you are not extracting a non-sclar value, the output will be
a single line per value.
Otherwise, you will either get a list of JSON or YAML values.

```bash,use=in-fixtures,exec
sy extract user.name user/password -o json < credentials.yml
```
