Tools are everything not directly related to managing secrets, and help to use them
while avoiding them to touch disk.

This can be achieved by putting the following capabilities together:

1. **Context Creation**
   * A context is just a bunch of properties in a structure.
   * Used to instantiate and customize templates.
   * Parts of it may be secret.
   * It can live in multiple places, such as files and in-memory as it is produced
     in real-time by programs. The latter can be 'sheesy' decrypting a file on the fly.
2. **Template Substitution**
   * Using a templating engine and a set of templates, the data can be placed in
     any kind of file to be consumed by other tools.
   * It's also useful to maintain a library of templates which are controlled by
     contexts, which change depending on the one use-case.


As an abstract example, this is how the build-pipeline for kubernetes could look like:

```bash
stage=production
merge \
    <(show-secret $stage/infrastructure.yml) \
    etc/team.json \
    etc/stages/$stage.yml \
| substitute \
    --separator $'---\n' \
    etc/template/k8s-shared-infrastructure.yml \
    etc/template/k8s-$stage-infrastructure.yml \
| kubectl --kubeconfig <(show-secret $stage/kube.config) apply -f -
```

**Read on to learn more about the individual tools to _merge_, _substitute_ and
_extract_**.
